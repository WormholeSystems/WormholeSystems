//! Import and export in the legacy-compatible file format: round trips, merge semantics,
//! and reading a file the legacy application wrote.

mod common;

use std::collections::HashMap;

use common::catalog::{K162, seed_catalog};
use common::{SYS_A, SYS_B, SYS_C, world};
use sqlx::PgPool;
use wormholesystems::allowlist::AffiliationAllowlist;
use wormholesystems::maps::connection::{AddConnection, add_connection};
use wormholesystems::maps::ignored::{AddIgnoredSystem, add_ignored_system, ignored_ids};
use wormholesystems::maps::signatures::{
    AddSignature, LinkSignature, add_signature, link_signature,
};
use wormholesystems::maps::solar_system::{
    AddSystem, SetHome, SetRally, add_system, set_home, set_rally,
};
use wormholesystems::maps::transfer::{
    SectionSet, export_map, import_map, import_map_as_new, parse_export,
};
use wormholesystems::maps::{
    ConnectionType, MapError, MassStatus, Role, SignatureGroup, SystemStatus, TimeStatus,
    WormholeSize,
};

fn all_sections() -> SectionSet {
    SectionSet {
        settings: true,
        access: true,
        solarsystems: true,
        connections: true,
        signatures: true,
        routes: true,
    }
}

/// A map with two placed systems, a linked wormhole between them, intel, and a home.
async fn populated_world(pool: &PgPool) -> common::World {
    seed_catalog(pool).await;
    let w = world(pool).await;
    let a = add_system(
        pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            x: 100.0,
            y: 200.0,
            alias: Some("HOME".into()),
        },
    )
    .await
    .unwrap();
    let b = add_system(
        pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
            x: 300.0,
            y: 200.0,
            alias: Some("1".into()),
        },
    )
    .await
    .unwrap();
    set_home(
        pool,
        w.owner,
        SetHome {
            map_id: w.map_id,
            map_solar_system_id: a.id,
            value: true,
        },
    )
    .await
    .unwrap();

    let conn = add_connection(
        pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: a.id,
            to_system: b.id,
            kind: ConnectionType::Wormhole,
            size: Some(WormholeSize::Medium),
        },
    )
    .await
    .unwrap();
    let sig = add_signature(
        pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Wormhole,
            signature_type_id: Some(K162),
            name: None,
            size: None,
            mass_status: Some(MassStatus::Reduced),
            time_status: Some(TimeStatus::Eol),
        },
    )
    .await
    .unwrap();
    link_signature(
        pool,
        w.owner,
        LinkSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
            connection_id: conn.id,
        },
    )
    .await
    .unwrap();

    sqlx::query(
        "insert into map_solar_system_details (map_id, solar_system_id, status, occupying_group, notes)
         values ($1, $2, 'hostile', 'Bad Corp', 'stay out')
         on conflict (map_id, solar_system_id)
         do update set status = 'hostile', occupying_group = 'Bad Corp', notes = 'stay out'",
    )
    .bind(w.map_id)
    .bind(SYS_C)
    .execute(pool)
    .await
    .unwrap();
    add_ignored_system(
        pool,
        w.owner,
        AddIgnoredSystem {
            map_id: w.map_id,
            solar_system_id: SYS_C,
        },
    )
    .await
    .unwrap();
    w
}

#[sqlx::test]
async fn a_round_trip_reproduces_the_map(pool: PgPool) {
    let w = populated_world(&pool).await;

    let file = export_map(&pool, w.owner, w.map_id, all_sections())
        .await
        .unwrap();
    let content = serde_json::to_string(&file).unwrap();
    let parsed = parse_export(&content, all_sections(), true).unwrap();
    let map = import_map_as_new(
        &pool,
        w.owner,
        &AffiliationAllowlist::default(),
        parsed,
        Some("Copy".into()),
    )
    .await
    .unwrap();

    assert_eq!(map.name, "Copy");
    let copied = wormholesystems::maps::map::get_map(
        &pool,
        w.owner,
        wormholesystems::maps::map::GetMap { map_id: map.id },
    )
    .await
    .unwrap();
    assert_eq!(copied.systems.len(), 2, "both placed systems came across");
    assert_eq!(copied.connections.len(), 1);
    let conn = &copied.connections[0];
    assert_eq!(conn.size, Some(WormholeSize::Medium));
    assert_eq!(conn.mass_status, Some(MassStatus::Reduced));
    assert_eq!(conn.time_status, Some(TimeStatus::Eol));

    // The linked signature came across, still linked and still typed.
    let sig = sqlx::query!(
        r#"select signature_id, signature_type_id, connection_id,
                  mass_status as "mass_status: MassStatus"
           from signatures where map_id = $1"#,
        map.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(sig.signature_id, "ABC-123");
    assert_eq!(sig.signature_type_id, Some(K162));
    assert_eq!(sig.connection_id, Some(conn.id));
    assert_eq!(sig.mass_status, Some(MassStatus::Reduced));

    // Intel without a placement survives as a details row without one.
    let details = sqlx::query!(
        r#"select status as "status: SystemStatus", occupying_group, notes
           from map_solar_system_details where map_id = $1 and solar_system_id = $2"#,
        map.id,
        SYS_C,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details.status, SystemStatus::Hostile);
    assert_eq!(details.occupying_group.as_deref(), Some("Bad Corp"));

    // Home followed the settings section.
    let home = sqlx::query_scalar!(
        "select solar_system_id from map_solar_systems where map_id = $1 and is_home",
        map.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(home, Some(SYS_A));

    // So did the ignore list, as its own rows on the new map.
    assert_eq!(ignored_ids(&pool, map.id).await.unwrap(), vec![SYS_C]);
}

#[sqlx::test]
async fn importing_twice_updates_rather_than_duplicates(pool: PgPool) {
    let w = populated_world(&pool).await;
    let file = export_map(&pool, w.owner, w.map_id, all_sections())
        .await
        .unwrap();
    let content = serde_json::to_string(&file).unwrap();
    let parsed = parse_export(&content, all_sections(), false).unwrap();

    let summary = import_map(&pool, w.owner, w.map_id, &parsed).await.unwrap();

    // Everything already exists under its natural key, so nothing is created; the one
    // connection is recognized as the same edge and skipped.
    assert_eq!(summary.systems.created, 0);
    assert_eq!(summary.systems.updated, 3);
    assert_eq!(summary.connections.created, 0);
    assert_eq!(summary.connections.skipped, 1);
    assert_eq!(summary.signatures.created, 0);
    assert_eq!(summary.signatures.updated, 1);

    let connections = sqlx::query_scalar!(
        "select count(*) from map_connections where map_id = $1",
        w.map_id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(connections, Some(1));
}

#[sqlx::test]
async fn a_legacy_file_imports_with_its_vocabulary_translated(pool: PgPool) {
    let w = populated_world(&pool).await;

    // As the legacy exporter writes it: `fresh`/`xlarge`/`healthy`, offset timestamps,
    // database booleans left as 0/1, and sections we did not ask for containing whatever
    // they like.
    let content = format!(
        r#"{{
        "format": "wormholesystems-map-export",
        "version": 1,
        "exported_at": "2026-08-20T10:00:00+00:00",
        "map_name": "Legacy Chain",
        "sections": {{
            "routes": "not even an array",
            "solarsystems": [
                {{"solarsystem_id": {SYS_A}, "alias": "L1", "position_x": 40, "position_y": 80,
                  "pinned": null, "status": "friendly", "occupier_alias": null, "notes": null}},
                {{"solarsystem_id": {SYS_B}, "alias": null, "position_x": 200, "position_y": 80,
                  "pinned": 1, "status": "unknown", "occupier_alias": null, "notes": null}},
                {{"solarsystem_id": 99999999, "alias": null, "position_x": 0, "position_y": 0,
                  "pinned": null, "status": "unknown", "occupier_alias": null, "notes": null}}
            ],
            "connections": [
                {{"from_solarsystem_id": {SYS_A}, "to_solarsystem_id": {SYS_B},
                  "wormhole": "K162", "type": "wormhole", "mass_status": "fresh",
                  "ship_size": "xlarge", "lifetime": "healthy",
                  "lifetime_updated_at": null,
                  "connected_at": "2026-08-19T22:15:00+02:00", "preserve_mass": 0}}
            ],
            "signatures": [
                {{"solarsystem_id": {SYS_B}, "signature_id": "XYZ-789", "category": "wormhole",
                  "type_name": "K162 - Unknown", "raw_type_name": null, "wormhole": "K162",
                  "connection_index": 0, "mass_status": "unknown", "ship_size": "frigate",
                  "lifetime": "eol", "lifetime_updated_at": "2026-08-20T09:00:00+00:00"}},
                {{"solarsystem_id": {SYS_B}, "signature_id": null, "category": "faction-warfare",
                  "type_name": null, "raw_type_name": "Weird Site", "wormhole": null,
                  "connection_index": null, "mass_status": null, "ship_size": null,
                  "lifetime": null, "lifetime_updated_at": null}}
            ]
        }}
    }}"#
    );

    let sections = SectionSet {
        solarsystems: true,
        connections: true,
        signatures: true,
        ..Default::default()
    };
    let parsed = parse_export(&content, sections, true).unwrap();
    let map = import_map_as_new(
        &pool,
        w.owner,
        &AffiliationAllowlist::default(),
        parsed,
        None,
    )
    .await
    .unwrap();
    assert_eq!(map.name, "Legacy Chain");

    let conn = sqlx::query!(
        r#"select mass_status as "mass_status: MassStatus",
                  time_status as "time_status: TimeStatus",
                  size as "size: WormholeSize", created_at
           from map_connections where map_id = $1"#,
        map.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    // Linking the signature fired the DB merge: the worst value in the group wins, so the
    // frigate-sized EOL scan overrides the connection's xlarge/healthy, and legacy's
    // `fresh` (with the sig's `unknown` skipped) lands as stable.
    assert_eq!(conn.size, Some(WormholeSize::Small));
    assert_eq!(conn.time_status, Some(TimeStatus::Eol));
    assert_eq!(conn.mass_status, Some(MassStatus::Stable));
    assert_eq!(
        conn.created_at.to_rfc3339(),
        "2026-08-19T20:15:00+00:00",
        "connected_at becomes the connection's age"
    );

    // The id-less legacy signature was skipped; the real one linked and kept its stamp.
    let sigs = sqlx::query!(
        r#"select signature_id, connection_id, size as "size: WormholeSize",
                  time_status_updated_at
           from signatures where map_id = $1"#,
        map.id,
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(sigs.len(), 1);
    assert_eq!(sigs[0].signature_id, "XYZ-789");
    assert!(sigs[0].connection_id.is_some());
    assert_eq!(sigs[0].size, Some(WormholeSize::Small));
    assert_eq!(
        sigs[0].time_status_updated_at.unwrap().to_rfc3339(),
        "2026-08-20T09:00:00+00:00",
        "the file's lifetime stamp survives the triggers"
    );

    // The unknown system fell out quietly, and the 0/1 database booleans read as booleans.
    let placed = sqlx::query!(
        r#"select solar_system_id as "solar_system_id!", is_pinned
           from map_solar_systems where map_id = $1 order by solar_system_id"#,
        map.id,
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(placed.len(), 2);
    assert!(
        placed
            .iter()
            .any(|p| p.solar_system_id == SYS_B && p.is_pinned)
    );
}

/// A legacy file with two placed systems and one wormhole between them, as legacy writes
/// it for the given `wormhole` and `ship_size` (both raw JSON).
fn legacy_connection_file(wormhole: &str, ship_size: &str) -> String {
    format!(
        r#"{{
        "format": "wormholesystems-map-export",
        "version": 1,
        "exported_at": "2026-10-01T10:00:00+00:00",
        "map_name": "Unsized",
        "sections": {{
            "solarsystems": [
                {{"solarsystem_id": {SYS_A}, "alias": null, "position_x": 0, "position_y": 0,
                  "pinned": false, "status": "unknown", "occupier_alias": null, "notes": null}},
                {{"solarsystem_id": {SYS_B}, "alias": null, "position_x": 200, "position_y": 0,
                  "pinned": false, "status": "unknown", "occupier_alias": null, "notes": null}}
            ],
            "connections": [
                {{"from_solarsystem_id": {SYS_A}, "to_solarsystem_id": {SYS_B},
                  "wormhole": {wormhole}, "type": "wormhole", "mass_status": "fresh",
                  "ship_size": {ship_size}, "lifetime": "healthy",
                  "lifetime_updated_at": null, "connected_at": null, "preserve_mass": false}}
            ]
        }}
    }}"#
    )
}

async fn import_connection_size(pool: &PgPool, content: &str) -> Option<WormholeSize> {
    let w = world(pool).await;
    let sections = SectionSet {
        solarsystems: true,
        connections: true,
        ..Default::default()
    };
    let parsed = parse_export(content, sections, true).unwrap();
    let map = import_map_as_new(
        pool,
        w.owner,
        &AffiliationAllowlist::default(),
        parsed,
        None,
    )
    .await
    .unwrap();
    sqlx::query_scalar!(
        r#"select size as "size: WormholeSize" from map_connections where map_id = $1"#,
        map.id,
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test]
async fn a_legacy_connection_without_a_size_imports_unsized(pool: PgPool) {
    let content = legacy_connection_file("null", "null");
    assert_eq!(import_connection_size(&pool, &content).await, None);
}

#[sqlx::test]
async fn the_owner_grant_is_never_touched(pool: PgPool) {
    let w = world(&pool).await;

    let content = r#"{
        "format": "wormholesystems-map-export",
        "version": 1,
        "exported_at": "2026-08-20T10:00:00+00:00",
        "map_name": "Grants",
        "sections": {
            "access": [
                {"entity_type": "character", "entity_id": 1001, "entity_name": "Char 1001",
                 "permission": "viewer", "expires_at": null},
                {"entity_type": "corporation", "entity_id": 7001, "entity_name": "New Corp",
                 "permission": "member", "expires_at": null}
            ]
        }
    }"#;
    let sections = SectionSet {
        access: true,
        ..Default::default()
    };
    let parsed = parse_export(content, sections, false).unwrap();
    let summary = import_map(&pool, w.owner, w.map_id, &parsed).await.unwrap();

    // The row demoting the owner to viewer is skipped; the new corp grant lands.
    assert_eq!(summary.access.skipped, 1);
    assert_eq!(summary.access.created, 1);
    let owner_role = sqlx::query_scalar!(
        r#"select role as "role: Role" from map_access
           where map_id = $1 and subject_id = 1001"#,
        w.map_id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(owner_role, Role::Owner);
}

#[sqlx::test]
async fn a_file_with_owner_grants_is_rejected(pool: PgPool) {
    let _ = world(&pool).await;
    let content = r#"{
        "format": "wormholesystems-map-export",
        "version": 1,
        "exported_at": "2026-08-20T10:00:00+00:00",
        "map_name": "Grants",
        "sections": {
            "access": [
                {"entity_type": "character", "entity_id": 5, "entity_name": null,
                 "permission": "owner", "expires_at": null}
            ]
        }
    }"#;
    let sections = SectionSet {
        access: true,
        ..Default::default()
    };
    assert!(matches!(
        parse_export(content, sections, false),
        Err(MapError::Validation(_))
    ));
}

#[sqlx::test]
async fn parse_refuses_wrong_files_with_a_reason(pool: PgPool) {
    let _ = pool; // parse is pure; the fixture just matches the harness.
    let sections = SectionSet {
        settings: true,
        ..Default::default()
    };

    for (content, expected) in [
        ("not json", "not valid JSON"),
        (
            r#"{"format": "something-else"}"#,
            "not a wormholesystems map export",
        ),
        (
            r#"{"format": "wormholesystems-map-export", "version": 2}"#,
            "incompatible version",
        ),
        (
            r#"{"format": "wormholesystems-map-export", "version": 1,
                "map_name": "x", "sections": {}}"#,
            "does not contain the \"settings\" section",
        ),
    ] {
        match parse_export(content, sections, false) {
            Err(MapError::Validation(msg)) => {
                assert!(
                    msg.contains(expected),
                    "{msg:?} should mention {expected:?}"
                )
            }
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    // A new map cannot take connections without the systems they hang between.
    let deps = SectionSet {
        connections: true,
        ..Default::default()
    };
    let content = r#"{"format": "wormholesystems-map-export", "version": 1,
        "map_name": "x", "sections": {"connections": []}}"#;
    assert!(parse_export(content, deps, false).is_ok());
    assert!(matches!(
        parse_export(content, deps, true),
        Err(MapError::Validation(_))
    ));
}

#[sqlx::test]
async fn importing_routes_into_a_new_map_replaces_the_seeded_watchlist(pool: PgPool) {
    let w = world(&pool).await;
    let content = format!(
        r#"{{
        "format": "wormholesystems-map-export",
        "version": 1,
        "exported_at": "2026-08-20T10:00:00+00:00",
        "map_name": "Routed",
        "sections": {{
            "routes": {{
                "route_solarsystems": [
                    {{"solarsystem_id": {SYS_B}, "is_pinned": true}}
                ],
                "ignored_solarsystems": [
                    {{"solarsystem_id": {SYS_C}}}
                ]
            }}
        }}
    }}"#
    );
    let sections = SectionSet {
        routes: true,
        ..Default::default()
    };
    let parsed = parse_export(&content, sections, true).unwrap();
    let map = import_map_as_new(
        &pool,
        w.owner,
        &AffiliationAllowlist::default(),
        parsed,
        None,
    )
    .await
    .unwrap();

    // Only the file's list: the trade-hub seeds made way for it.
    let entries = sqlx::query!(
        "select solar_system_id, is_pinned from map_watchlist where map_id = $1",
        map.id,
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].solar_system_id, SYS_B);
    assert!(entries[0].is_pinned);
    assert_eq!(ignored_ids(&pool, map.id).await.unwrap(), vec![SYS_C]);

    // Importing the same file again leaves one row, counted as updated rather than made.
    let parsed = parse_export(&content, sections, true).unwrap();
    let counts = import_map(&pool, w.owner, map.id, &parsed).await.unwrap();
    assert_eq!(counts.routes.updated, 2);
    assert_eq!(counts.routes.created, 0);
    assert_eq!(ignored_ids(&pool, map.id).await.unwrap(), vec![SYS_C]);
}

#[sqlx::test]
async fn export_needs_manager_and_members_are_refused(pool: PgPool) {
    let w = world(&pool).await;
    let member = common::member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;
    assert!(matches!(
        export_map(&pool, member, w.map_id, all_sections()).await,
        Err(MapError::Forbidden)
    ));
}

#[sqlx::test]
async fn a_faction_warfare_signature_keeps_the_legacy_code_on_the_wire(pool: PgPool) {
    let w = populated_world(&pool).await;
    add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "MSA-264".into(),
            group: SignatureGroup::FactionWarfare,
            name: Some("Amarr Moderate NVY-3".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let file = export_map(&pool, w.owner, w.map_id, all_sections())
        .await
        .unwrap();
    let row = file
        .sections
        .signatures
        .as_deref()
        .unwrap_or_default()
        .iter()
        .find(|s| s.signature_id.as_deref() == Some("MSA-264"))
        .unwrap();
    assert_eq!(
        row.category.as_deref(),
        Some("faction-warfare"),
        "legacy spells the category with a hyphen, and reads it back that way"
    );

    let content = serde_json::to_string(&file).unwrap();
    let parsed = parse_export(&content, all_sections(), true).unwrap();
    let map = import_map_as_new(
        &pool,
        w.owner,
        &AffiliationAllowlist::default(),
        parsed,
        None,
    )
    .await
    .unwrap();
    let group = sqlx::query_scalar!(
        r#"select "group" as "group!: SignatureGroup" from signatures
           where map_id = $1 and signature_id = 'MSA-264'"#,
        map.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(group, SignatureGroup::FactionWarfare);
}

/// Every rally point goes out, oldest first, under the list key alone.
#[sqlx::test]
async fn export_carries_every_rally_point(pool: PgPool) {
    let w = populated_world(&pool).await;
    let placed = placements_by_system(&pool, w.map_id).await;
    for sys in [SYS_B, SYS_A] {
        set_rally(
            &pool,
            w.owner,
            SetRally {
                map_id: w.map_id,
                map_solar_system_id: placed[&sys],
                value: true,
            },
        )
        .await
        .unwrap();
    }

    let file = export_map(&pool, w.owner, w.map_id, all_sections())
        .await
        .unwrap();
    let json = serde_json::to_value(&file).unwrap();
    let settings = &json["sections"]["settings"];
    assert_eq!(
        settings["rally_solarsystem_ids"],
        serde_json::json!([SYS_B, SYS_A])
    );
    assert!(settings.get("rally_solarsystem_id").is_none());

    // And they come back in the same order on the copy.
    let parsed = parse_export(&json.to_string(), all_sections(), true).unwrap();
    let map = import_map_as_new(
        &pool,
        w.owner,
        &AffiliationAllowlist::default(),
        parsed,
        None,
    )
    .await
    .unwrap();
    assert_eq!(rally_systems(&pool, map.id).await, vec![SYS_B, SYS_A]);
}

/// The list as written, the single id older files carry, unknown or unplaced ids dropped,
/// and only the newest two kept.
#[sqlx::test]
async fn import_reads_rally_points_in_either_spelling(pool: PgPool) {
    let w = populated_world(&pool).await;
    add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_C,
            x: 500.0,
            y: 200.0,
            alias: None,
        },
    )
    .await
    .unwrap();
    let sections = SectionSet {
        settings: true,
        ..Default::default()
    };

    for (rally, expected) in [
        (
            format!(r#""rally_solarsystem_ids": [{SYS_A}, {SYS_B}]"#),
            vec![SYS_A, SYS_B],
        ),
        (format!(r#""rally_solarsystem_id": {SYS_B}"#), vec![SYS_B]),
        (
            format!(r#""rally_solarsystem_ids": [99999999, {SYS_C}]"#),
            vec![SYS_C],
        ),
        (
            format!(r#""rally_solarsystem_ids": [{SYS_A}, {SYS_B}, {SYS_C}]"#),
            vec![SYS_B, SYS_C],
        ),
        (r#""rally_solarsystem_ids": null"#.to_string(), vec![]),
        (r#""home_solarsystem_id": null"#.to_string(), vec![]),
    ] {
        let parsed = parse_export(&rally_settings_file(&rally), sections, false).unwrap();
        import_map(&pool, w.owner, w.map_id, &parsed).await.unwrap();
        assert_eq!(rally_systems(&pool, w.map_id).await, expected, "{rally}");
    }

    let malformed = rally_settings_file(r#""rally_solarsystem_ids": ["abc"]"#);
    assert!(matches!(
        parse_export(&malformed, sections, false),
        Err(MapError::Validation(_))
    ));
}

fn rally_settings_file(rally: &str) -> String {
    format!(
        r#"{{
        "format": "wormholesystems-map-export",
        "version": 1,
        "exported_at": "2026-10-04T10:00:00+00:00",
        "map_name": "Rally Chain",
        "sections": {{
            "settings": {{
                "name": "Rally Chain", "layout": "manual", "allow_layout_override": true,
                "constant_width_enabled": false, "bookmark_format_wormhole": "{{alias}}",
                "bookmark_format_kspace": "{{name}}", "bookmark_format_return": "return",
                "bookmark_alias_scheme": "numeric", "bookmark_ignored_alias": null,
                "home_solarsystem_id": null, {rally}
            }}
        }}
    }}"#
    )
}

async fn placements_by_system(pool: &PgPool, map_id: i64) -> HashMap<i64, i64> {
    sqlx::query!(
        r#"select id, solar_system_id as "solar_system_id!" from map_solar_systems
           where map_id = $1 and solar_system_id is not null"#,
        map_id,
    )
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|r| (r.solar_system_id, r.id))
    .collect()
}

/// The map's rally systems, oldest first.
async fn rally_systems(pool: &PgPool, map_id: i64) -> Vec<i64> {
    sqlx::query_scalar!(
        r#"select solar_system_id as "solar_system_id!" from map_solar_systems
           where map_id = $1 and is_rally order by rallied_at, id"#,
        map_id,
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Legacy writes the in-game "closure imminent" state as `expired`; it has to survive the
/// trip out and back in.
#[sqlx::test]
async fn an_expired_hole_round_trips_as_expired(pool: PgPool) {
    use wormholesystems::maps::connection::{SetConnectionStatus, set_connection_status};

    let w = populated_world(&pool).await;
    let connection_id =
        sqlx::query_scalar!("select id from map_connections where map_id = $1", w.map_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id,
            time_status: Some(Some(TimeStatus::Expired)),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let file = export_map(&pool, w.owner, w.map_id, all_sections())
        .await
        .unwrap();
    let content = serde_json::to_string(&file).unwrap();
    assert!(content.contains(r#""lifetime":"expired""#));

    let parsed = parse_export(&content, all_sections(), true).unwrap();
    let map = import_map_as_new(
        &pool,
        w.owner,
        &AffiliationAllowlist::default(),
        parsed,
        None,
    )
    .await
    .unwrap();
    let status = sqlx::query_scalar!(
        r#"select time_status as "time_status: TimeStatus" from map_connections
           where map_id = $1"#,
        map.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, Some(TimeStatus::Expired));
}
