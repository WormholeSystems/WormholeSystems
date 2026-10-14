//! Undo and redo, command by command: every undoable change, run on a map of its own, then
//! undone, redone and undone again, must leave the map's tables exactly as they were each
//! time. A command whose inverse forgets a column shows up here as a diff rather than as a
//! report from somebody whose hole came back a day younger.

mod common;

use common::catalog::{THERA, seed_catalog, seed_thera};
use common::{SYS_A, SYS_B, SYS_C, world};
use serde_json::{Value, json};
use sqlx::PgPool;
use wormholesystems::allowlist::AffiliationAllowlist;
use wormholesystems::maps::events_log::{MapIdBody, redo, undo};
use wormholesystems::maps::map::{CreateMap, UpdateMap, create_map, update_map};
use wormholesystems::maps::{Actor, CommandOutput, MapCommand, execute};

/// Two more systems than the shared universe has: one for a stale edge to lead to, one to
/// sit on the ignore list.
const SYS_D: i64 = 30000146;
const SYS_E: i64 = 30000147;

/// Every table a map command writes, as JSON in id order, less what an undo may change:
///
/// - `updated_at` says when a row was last touched, and an undo touches it.
/// - `time_status_updated_at`, for the edits only (`$2`): the stamp trigger marks a lifetime
///   change as made now, undo included, and the edit commands carry no old stamp to put
///   back. A removal does, so there it is compared.
/// - The ignore list by system rather than by id: an undone removal comes back as a new
///   row, which nothing names by id.
/// - A details row holding nothing but the defaults says the same as no row at all.
const TABLES: [(&str, &str); 7] = [
    (
        "map_solar_systems",
        "select coalesce(jsonb_agg(to_jsonb(t) - 'updated_at' order by t.id), '[]')
         from map_solar_systems t where map_id = $1 and $2 in (true, false)",
    ),
    (
        "map_connections",
        "select coalesce(jsonb_agg(to_jsonb(t) - 'updated_at'
                    - case when $2 then 'time_status_updated_at' else '' end
                    order by t.id), '[]')
         from map_connections t where map_id = $1",
    ),
    (
        "signatures",
        "select coalesce(jsonb_agg(to_jsonb(t) - 'updated_at'
                    - case when $2 then 'time_status_updated_at' else '' end
                    order by t.id), '[]')
         from signatures t where map_id = $1",
    ),
    (
        "map_solar_system_details",
        "select coalesce(jsonb_agg(to_jsonb(t) - 'updated_at' order by t.id), '[]')
         from map_solar_system_details t
         where map_id = $1 and $2 in (true, false)
           and (status <> 'unknown' or occupying_group is not null or notes is not null)",
    ),
    (
        "map_connection_jumps",
        "select coalesce(jsonb_agg(to_jsonb(t) - 'updated_at' order by t.id), '[]')
         from map_connection_jumps t where map_id = $1 and $2 in (true, false)",
    ),
    (
        "map_watchlist",
        "select coalesce(jsonb_agg(to_jsonb(t) order by t.id), '[]')
         from map_watchlist t where map_id = $1 and $2 in (true, false)",
    ),
    (
        "map_ignored_solar_systems",
        "select coalesce(jsonb_agg(jsonb_build_object('id', t.solar_system_id)
                    order by t.solar_system_id), '[]')
         from map_ignored_solar_systems t where map_id = $1 and $2 in (true, false)",
    ),
];

async fn tables(pool: &PgPool, map_id: i64, ignore_stamp: bool) -> Vec<(&'static str, Value)> {
    let mut out = Vec::new();
    for (name, sql) in TABLES {
        let rows: Value = sqlx::query_scalar(sql)
            .bind(map_id)
            .bind(ignore_stamp)
            .fetch_one(pool)
            .await
            .unwrap();
        out.push((name, rows));
    }
    out
}

/// What differs between two snapshots, row by row and field by field, so a failure says
/// which column an inverse forgot rather than printing both tables.
fn differences(
    case: &str,
    when: &str,
    want: &[(&str, Value)],
    got: &[(&str, Value)],
) -> Vec<String> {
    let rows = |v: &Value| -> Vec<(i64, Value)> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|r| (r["id"].as_i64().unwrap(), r.clone()))
            .collect()
    };
    let mut out = Vec::new();
    for ((table, want), (_, got)) in want.iter().zip(got) {
        let (want, got) = (rows(want), rows(got));
        for (id, w) in &want {
            let Some((_, g)) = got.iter().find(|(g, _)| g == id) else {
                out.push(format!("{case}: {table} #{id} missing {when}"));
                continue;
            };
            for (field, value) in w.as_object().unwrap() {
                if g[field] != *value {
                    out.push(format!(
                        "{case}: {table} #{id}.{field} {when}: want {value}, got {}",
                        g[field]
                    ));
                }
            }
        }
        for (id, _) in got
            .iter()
            .filter(|(g, _)| !want.iter().any(|(w, _)| w == g))
        {
            out.push(format!("{case}: {table} #{id} extra {when}"));
        }
    }
    out
}

async fn apply(pool: &PgPool, actor: Actor, cmd: Value) -> Result<CommandOutput, String> {
    let cmd: MapCommand = serde_json::from_value(cmd).map_err(|e| e.to_string())?;
    execute(pool, actor, cmd)
        .await
        .map_err(|e| format!("{e:?}"))
}

async fn head(pool: &PgPool, map_id: i64) -> Option<i64> {
    sqlx::query_scalar("select head_event_id from maps where id = $1")
        .bind(map_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn id_of(pool: &PgPool, sql: &'static str, map_id: i64) -> i64 {
    sqlx::query_scalar(sql)
        .bind(map_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// What every case runs against: two placed systems joined by a wormhole with a scan on
/// the near side and a jump through it, a third with a stale edge to it, a loose wormhole
/// scan on either side, a data site, the seeded watchlist and an ignored system.
struct Scene {
    map_id: i64,
    a: i64,
    b: i64,
    conn: i64,
    linked: i64,
    loose: i64,
    far: i64,
    data: i64,
    jump: i64,
    watch: i64,
    /// The node the loose scan in the first system raised, when the map draws them.
    ghost: Option<i64>,
}

async fn seed_extra_systems(pool: &PgPool) {
    for (id, name) in [(SYS_D, "Fourth"), (SYS_E, "Fifth")] {
        sqlx::query(
            "insert into solar_systems (id, constellation_id, region_id, name, security_status)
             values ($1, 20000001, 10000001, $2, 0.5)",
        )
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
    }
}

/// Build the scene on a fresh map, so no case sees what another left behind.
async fn scene(pool: &PgPool, actor: Actor, ghosting: bool) -> Scene {
    let map_id = create_map(
        pool,
        actor,
        &AffiliationAllowlist::default(),
        CreateMap {
            name: "Round trip".into(),
            description: None,
        },
    )
    .await
    .unwrap()
    .id;
    let run = |cmd: Value| async move { apply(pool, actor, cmd).await.unwrap() };

    let mut placed = Vec::new();
    for (system, x) in [(SYS_A, 0.0), (SYS_B, 200.0), (SYS_D, 400.0)] {
        let out = run(json!({"command": "add_system", "map_id": map_id,
                             "solar_system_id": system, "x": x, "y": 0.0, "alias": null}))
        .await;
        let CommandOutput::System(system) = out else {
            panic!("{out:?}")
        };
        placed.push(system.id);
    }
    let (a, b, d) = (placed[0], placed[1], placed[2]);
    let mut edges = Vec::new();
    for (from, to) in [(a, b), (b, d)] {
        let out = run(json!({"command": "add_connection", "map_id": map_id,
                             "from_system": from, "to_system": to, "kind": "wormhole"}))
        .await;
        let CommandOutput::Connection(connection) = out else {
            panic!("{out:?}")
        };
        edges.push(connection.id);
    }
    let (conn, stale) = (edges[0], edges[1]);
    // Critical for two hours: what the stale sweep takes. Two writes, because marking the
    // lifetime stamps it now.
    for sql in [
        "update map_connections set time_status = 'critical' where id = $1",
        "update map_connections set time_status_updated_at = now() - interval '2 hours'
         where id = $1",
    ] {
        sqlx::query(sql).bind(stale).execute(pool).await.unwrap();
    }

    let mut scans = Vec::new();
    for (system, sig, group) in [
        (SYS_A, "ABC-123", "wormhole"),
        (SYS_A, "XYZ-555", "wormhole"),
        (SYS_B, "FAR-777", "wormhole"),
        (SYS_B, "DAT-001", "data"),
    ] {
        let out = run(json!({"command": "add_signature", "map_id": map_id,
                             "solar_system_id": system, "signature_id": sig, "group": group}))
        .await;
        let CommandOutput::Signature(signature) = out else {
            panic!("{out:?}")
        };
        scans.push(signature.id);
    }
    let (linked, loose, far, data) = (scans[0], scans[1], scans[2], scans[3]);
    run(
        json!({"command": "link_signature", "map_id": map_id, "signature_pk": linked,
               "connection_id": conn}),
    )
    .await;
    run(
        json!({"command": "add_connection_jump", "map_id": map_id, "connection_id": conn,
               "direction": "outbound", "ship_type_id": null, "mass": 20000000}),
    )
    .await;
    run(json!({"command": "add_ignored_system", "map_id": map_id, "solar_system_id": SYS_E})).await;
    let jump = id_of(
        pool,
        "select id from map_connection_jumps where map_id = $1",
        map_id,
    )
    .await;
    let watch = id_of(
        pool,
        "select min(id) from map_watchlist where map_id = $1",
        map_id,
    )
    .await;

    let mut ghost = None;
    if ghosting {
        update_map(
            pool,
            actor,
            UpdateMap {
                map_id,
                ghost_unlinked_wormholes: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        ghost = Some(
            id_of(
                pool,
                "select min(id) from map_solar_systems
                 where map_id = $1 and solar_system_id is null",
                map_id,
            )
            .await,
        );
    }
    Scene {
        map_id,
        a,
        b,
        conn,
        linked,
        loose,
        far,
        data,
        jump,
        watch,
        ghost,
    }
}

/// The command a case runs, built against its scene.
type Command = fn(&Scene) -> Value;

/// A case: its name and its command.
type Case = (&'static str, Command);

/// One case on a scene of its own: apply, undo, redo, and undo again. Returns everything
/// that came out different.
async fn check(
    pool: &PgPool,
    actor: Actor,
    ghosting: bool,
    case: &str,
    cmd: &Command,
) -> Vec<String> {
    let s = scene(pool, actor, ghosting).await;
    let body = MapIdBody { map_id: s.map_id };
    // An edit's undo re-stamps the lifetime (see `TABLES`).
    let ignore_stamp =
        case.starts_with("set_connection_status") || case.starts_with("update_signature");
    let snap = || tables(pool, s.map_id, ignore_stamp);

    let before = snap().await;
    let head_before = head(pool, s.map_id).await;
    if let Err(e) = apply(pool, actor, cmd(&s)).await {
        return vec![format!("{case}: {e}")];
    }
    if head(pool, s.map_id).await == head_before {
        return vec![format!("{case}: changed nothing undoable")];
    }
    let after = snap().await;

    let mut failures = Vec::new();
    let steps = [
        ("after undo", &before),
        ("after redo", &after),
        ("after the second undo", &before),
    ];
    for (i, (when, want)) in steps.into_iter().enumerate() {
        let moved = if i == 1 {
            redo(pool, actor, body.clone()).await
        } else {
            undo(pool, actor, body.clone()).await
        };
        if let Err(e) = moved {
            failures.push(format!("{case}: failed {when}: {e:?}"));
            break;
        }
        failures.extend(differences(case, when, want, &snap().await));
    }
    failures
}

/// Run every case. One listed in [`KNOWN_GAPS`] is expected to fail, and has to come off
/// the list once it no longer does, so a fix cannot go unnoticed either.
async fn round_trip(pool: &PgPool, ghosting: bool, cases: &[Case]) {
    let w = world(pool).await;
    seed_extra_systems(pool).await;
    seed_catalog(pool).await;
    seed_thera(pool).await;
    let mut failures = Vec::new();
    for (case, cmd) in cases {
        let found = check(pool, w.owner, ghosting, case, cmd).await;
        let known = KNOWN_GAPS.iter().any(|(gap, _)| gap == case);
        match (known, found.is_empty()) {
            (false, _) => failures.extend(found),
            (true, true) => {
                failures.push(format!("{case}: round-trips now, take it off KNOWN_GAPS"))
            }
            (true, false) => {}
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Cases whose undo or redo does not yet put the map back, and what it gets wrong.
const KNOWN_GAPS: &[(&str, &str)] = &[(
    "update_signature away from a wormhole",
    "undo brings the ghost back but not the scan's link to it",
)];

const CASES: &[Case] = &[
    ("add_system", |s| {
        json!({"command": "add_system", "map_id": s.map_id, "solar_system_id": SYS_C,
               "x": 600.0, "y": 0.0, "alias": "C"})
    }),
    (
        "remove_system",
        |s| json!({"command": "remove_system", "map_id": s.map_id, "map_solar_system_id": s.b}),
    ),
    ("remove_systems", |s| {
        json!({"command": "remove_systems", "map_id": s.map_id,
               "map_solar_system_ids": [s.a, s.b]})
    }),
    (
        "clear_map",
        |s| json!({"command": "clear_map", "map_id": s.map_id}),
    ),
    ("move_system", |s| {
        json!({"command": "move_system", "map_id": s.map_id, "map_solar_system_id": s.a,
               "x": 40.0, "y": 80.0})
    }),
    ("move_systems", |s| {
        json!({"command": "move_systems", "map_id": s.map_id, "moves": [
            {"map_solar_system_id": s.a, "x": 40.0, "y": 80.0},
            {"map_solar_system_id": s.b, "x": 240.0, "y": 80.0}]})
    }),
    ("set_alias", |s| {
        json!({"command": "set_alias", "map_id": s.map_id, "map_solar_system_id": s.a,
               "alias": "Home"})
    }),
    ("set_status", |s| {
        json!({"command": "set_status", "map_id": s.map_id, "map_solar_system_id": s.a,
               "status": "hostile"})
    }),
    ("set_occupier", |s| {
        json!({"command": "set_occupier", "map_id": s.map_id, "map_solar_system_id": s.a,
               "occupier": "Hard Knocks"})
    }),
    ("set_notes", |s| {
        json!({"command": "set_notes", "map_id": s.map_id, "map_solar_system_id": s.a,
               "notes": "rolled at downtime"})
    }),
    ("set_home", |s| {
        json!({"command": "set_home", "map_id": s.map_id, "map_solar_system_id": s.a,
               "value": true})
    }),
    ("set_rally", |s| {
        json!({"command": "set_rally", "map_id": s.map_id, "map_solar_system_id": s.b,
               "value": true})
    }),
    ("set_pinned", |s| {
        json!({"command": "set_pinned", "map_id": s.map_id, "map_solar_system_id": s.b,
               "value": true})
    }),
    ("add_connection", |s| {
        json!({"command": "add_connection", "map_id": s.map_id, "from_system": s.a,
               "to_system": s.b, "kind": "stargate"})
    }),
    ("set_connection_status", |s| {
        json!({"command": "set_connection_status", "map_id": s.map_id,
               "connection_id": s.conn, "mass_status": "reduced", "time_status": "eol",
               "size": "large"})
    }),
    (
        "remove_connection",
        |s| json!({"command": "remove_connection", "map_id": s.map_id, "connection_id": s.conn}),
    ),
    (
        "clean_stale_connections",
        |s| json!({"command": "clean_stale_connections", "map_id": s.map_id}),
    ),
    ("track_jump", |s| {
        json!({"command": "track_jump", "map_id": s.map_id, "from_map_solar_system_id": s.a,
               "to_solar_system_id": SYS_C, "x": 0.0, "y": 100.0, "signature_pk": s.loose})
    }),
    ("add_signature", |s| {
        json!({"command": "add_signature", "map_id": s.map_id, "solar_system_id": SYS_B,
               "signature_id": "NEW-001", "group": "wormhole", "time_status": "eol"})
    }),
    ("update_signature", |s| {
        json!({"command": "update_signature", "map_id": s.map_id, "signature_pk": s.linked,
               "name": "C247", "mass_status": "critical", "time_status": "eol"})
    }),
    (
        "remove_signature",
        |s| json!({"command": "remove_signature", "map_id": s.map_id, "signature_pk": s.linked}),
    ),
    ("remove_signatures", |s| {
        json!({"command": "remove_signatures", "map_id": s.map_id,
               "signature_pks": [s.linked, s.data]})
    }),
    ("paste_signatures", |s| {
        json!({"command": "paste_signatures", "map_id": s.map_id, "solar_system_id": SYS_B,
               "signatures": [
                   {"signature_id": "DAT-001", "group": "relic"},
                   {"signature_id": "GAS-002", "group": "gas"}]})
    }),
    ("link_signature", |s| {
        json!({"command": "link_signature", "map_id": s.map_id, "signature_pk": s.far,
               "connection_id": s.conn})
    }),
    ("link_signature displacing another", |s| {
        json!({"command": "link_signature", "map_id": s.map_id, "signature_pk": s.loose,
               "connection_id": s.conn})
    }),
    (
        "unlink_signature",
        |s| json!({"command": "unlink_signature", "map_id": s.map_id, "signature_pk": s.linked}),
    ),
    ("add_connection_jump", |s| {
        json!({"command": "add_connection_jump", "map_id": s.map_id,
               "connection_id": s.conn, "direction": "inbound", "ship_type_id": null,
               "mass": 300000000})
    }),
    ("update_connection_jump", |s| {
        json!({"command": "update_connection_jump", "map_id": s.map_id, "jump_pk": s.jump,
               "direction": "inbound", "mass": 100000000})
    }),
    (
        "remove_connection_jump",
        |s| json!({"command": "remove_connection_jump", "map_id": s.map_id, "jump_pk": s.jump}),
    ),
    (
        "add_watchlist_entry",
        |s| json!({"command": "add_watchlist_entry", "map_id": s.map_id, "solar_system_id": SYS_D}),
    ),
    ("set_watchlist_pinned", |s| {
        json!({"command": "set_watchlist_pinned", "map_id": s.map_id, "entry_id": s.watch,
               "value": false})
    }),
    (
        "remove_watchlist_entry",
        |s| json!({"command": "remove_watchlist_entry", "map_id": s.map_id, "entry_id": s.watch}),
    ),
    (
        "add_ignored_system",
        |s| json!({"command": "add_ignored_system", "map_id": s.map_id, "solar_system_id": SYS_C}),
    ),
    ("remove_ignored_system", |s| {
        json!({"command": "remove_ignored_system", "map_id": s.map_id,
               "solar_system_id": SYS_E})
    }),
    ("add_eve_scout_connections", |s| {
        json!({"command": "add_eve_scout_connections", "map_id": s.map_id,
               "hub_solar_system_id": THERA, "at": null, "holes": [
                   {"solar_system_id": SYS_B, "hub_signature": "THE-001",
                    "signature": "FAR-002", "wormhole_type": "C247",
                    "mass_status": "reduced", "time_status": "eol", "size": "medium"},
                   {"solar_system_id": SYS_C, "hub_signature": "THE-002",
                    "signature": "OUT-003", "wormhole_type": "C247",
                    "mass_status": "stable", "time_status": "stable", "size": null}]})
    }),
    (
        "clear_ignored_systems",
        |s| json!({"command": "clear_ignored_systems", "map_id": s.map_id}),
    ),
];

/// The same, on a map that draws the holes nobody has been through: the commands that only
/// exist for ghosts, and the ones whose undo has ghosts to put back.
const GHOST_CASES: &[Case] = &[
    ("add_signature raising a ghost", |s| {
        json!({"command": "add_signature", "map_id": s.map_id, "solar_system_id": SYS_B,
               "signature_id": "NEW-001", "group": "wormhole"})
    }),
    ("resolve_ghost_system", |s| {
        json!({"command": "resolve_ghost_system", "map_id": s.map_id,
               "map_solar_system_id": s.ghost, "solar_system_id": SYS_C,
               "mass_status": "reduced"})
    }),
    ("resolve_ghost_system onto a placed system", |s| {
        json!({"command": "resolve_ghost_system", "map_id": s.map_id,
               "map_solar_system_id": s.ghost, "solar_system_id": SYS_B})
    }),
    (
        "remove_system with its ghost",
        |s| json!({"command": "remove_system", "map_id": s.map_id, "map_solar_system_id": s.a}),
    ),
    (
        "remove_signature behind a ghost",
        |s| json!({"command": "remove_signature", "map_id": s.map_id, "signature_pk": s.loose}),
    ),
    ("update_signature away from a wormhole", |s| {
        json!({"command": "update_signature", "map_id": s.map_id, "signature_pk": s.loose,
               "group": "data"})
    }),
    ("track_jump through a ghost", |s| {
        json!({"command": "track_jump", "map_id": s.map_id, "from_map_solar_system_id": s.a,
               "to_solar_system_id": SYS_C, "x": 0.0, "y": 100.0, "signature_pk": s.loose})
    }),
];

#[sqlx::test]
async fn every_command_undoes_and_redoes_to_the_same_tables(pool: PgPool) {
    round_trip(&pool, false, CASES).await;
}

#[sqlx::test]
async fn every_ghost_command_undoes_and_redoes_to_the_same_tables(pool: PgPool) {
    round_trip(&pool, true, GHOST_CASES).await;
}
