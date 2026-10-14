//! Connection jump tracking: manual CRUD, automatic transit capture, claiming, pruning.

mod common;

use common::{SYS_A, SYS_B, SYS_C, member_with_role, world};
use sqlx::PgPool;
use wormholesystems::maps::connection::{AddConnection, add_connection};
use wormholesystems::maps::jumps::{
    AddConnectionJump, JumpDirection, RemoveConnectionJump, UpdateConnectionJump, add_jump,
    claim_pending, list_jumps, prune_unclaimed, record_transit, remove_jump, update_jump,
};
use wormholesystems::maps::solar_system::{AddSystem, add_system};
use wormholesystems::maps::{Actor, ConnectionType, MapError, Role};

async fn place(pool: &PgPool, actor: Actor, map_id: i64, system: i64) -> i64 {
    add_system(
        pool,
        actor,
        AddSystem {
            map_id,
            solar_system_id: system,
            x: 0.0,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// A ship type (SDE category 6 → group → type with a hull mass).
async fn seed_ship(pool: &PgPool, type_id: i64, name: &str, mass: f64) {
    sqlx::query("insert into categories (id, name, published) values (6, 'Ship', true) on conflict do nothing")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "insert into groups (id, category_id, name, published) values (25, 6, 'Frigate', true)
         on conflict do nothing",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into types (id, group_id, name, published, mass) values ($1, 25, $2, true, $3)",
    )
    .bind(type_id)
    .bind(name)
    .bind(mass)
    .execute(pool)
    .await
    .unwrap();
}

async fn wormhole_between(pool: &PgPool, actor: Actor, map_id: i64, a: i64, b: i64) -> i64 {
    add_connection(
        pool,
        actor,
        AddConnection {
            map_id,
            from_system: a,
            to_system: b,
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// Let the owner share their movements on the map.
async fn allow_tracking(pool: &PgPool, map_id: i64, user_id: i64) {
    sqlx::query(
        "insert into map_user_settings (map_id, user_id, tracking_allowed) values ($1, $2, true)
         on conflict (map_id, user_id) do update set tracking_allowed = true",
    )
    .bind(map_id)
    .bind(user_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn set_ship(pool: &PgPool, character_id: i64, ship_type_id: i64) {
    sqlx::query(
        "insert into character_status (character_id, online, ship_type_id)
         values ($1, true, $2)
         on conflict (character_id) do update set ship_type_id = excluded.ship_type_id",
    )
    .bind(character_id)
    .bind(ship_type_id)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn manual_jump_crud(pool: PgPool) {
    let w = world(&pool).await;
    seed_ship(&pool, 587, "Rifter", 1_067_000.0).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let conn = wormhole_between(&pool, w.owner, w.map_id, a, b).await;

    // Neither mass nor ship type → Validation; viewers can't log at all.
    let err = add_jump(
        &pool,
        w.owner,
        AddConnectionJump {
            map_id: w.map_id,
            connection_id: conn,
            direction: JumpDirection::Outbound,
            ship_type_id: None,
            mass: None,
        },
    )
    .await;
    assert!(matches!(err, Err(MapError::Validation(_))));
    let viewer = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Viewer).await;
    let err = add_jump(
        &pool,
        viewer,
        AddConnectionJump {
            map_id: w.map_id,
            connection_id: conn,
            direction: JumpDirection::Outbound,
            ship_type_id: Some(587),
            mass: None,
        },
    )
    .await;
    assert!(matches!(err, Err(MapError::Forbidden)));

    // Ship type without a mass derives the hull mass; outbound = from → to.
    let jump = add_jump(
        &pool,
        w.owner,
        AddConnectionJump {
            map_id: w.map_id,
            connection_id: conn,
            direction: JumpDirection::Outbound,
            ship_type_id: Some(587),
            mass: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(jump.mass, 1_067_000);
    assert!(jump.is_manual);
    assert_eq!(jump.from_solar_system_id, SYS_A);
    assert_eq!(jump.to_solar_system_id, SYS_B);
    assert_eq!(jump.ship_type_name.as_deref(), Some("Rifter"));

    // An explicit mass wins over the ship type; inbound flips the endpoints.
    let heavy = add_jump(
        &pool,
        w.owner,
        AddConnectionJump {
            map_id: w.map_id,
            connection_id: conn,
            direction: JumpDirection::Inbound,
            ship_type_id: Some(587),
            mass: Some(300_000_000),
        },
    )
    .await
    .unwrap();
    assert_eq!(heavy.mass, 300_000_000);
    assert_eq!(heavy.from_solar_system_id, SYS_B);
    assert_eq!(heavy.to_solar_system_id, SYS_A);

    // Update: direction flip alone keeps the mass; a ship change re-derives it.
    let flipped = update_jump(
        &pool,
        w.owner,
        UpdateConnectionJump {
            map_id: w.map_id,
            jump_pk: heavy.id,
            direction: Some(JumpDirection::Outbound),
            ship_type_id: None,
            mass: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(flipped.from_solar_system_id, SYS_A);
    assert_eq!(flipped.mass, 300_000_000);
    let rederived = update_jump(
        &pool,
        w.owner,
        UpdateConnectionJump {
            map_id: w.map_id,
            jump_pk: heavy.id,
            direction: None,
            ship_type_id: Some(Some(587)),
            mass: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(rederived.mass, 1_067_000);

    // The log lists newest first; aggregates land on the connection payload.
    let jumps = list_jumps(&pool, w.owner, w.map_id, conn).await.unwrap();
    assert_eq!(jumps.len(), 2);
    assert_eq!(jumps[0].id, heavy.id);
    let agg = sqlx::query!(
        r#"select count(*) as "count!", coalesce(sum(mass), 0)::bigint as "sum!"
           from map_connection_jumps where connection_id = $1"#,
        conn,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(agg.count, 2);
    assert_eq!(agg.sum, 1_067_000 * 2);

    remove_jump(
        &pool,
        w.owner,
        RemoveConnectionJump {
            map_id: w.map_id,
            jump_pk: jump.id,
        },
    )
    .await
    .unwrap();
    assert_eq!(
        list_jumps(&pool, w.owner, w.map_id, conn)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[sqlx::test]
async fn transit_capture_and_claiming(pool: PgPool) {
    let w = world(&pool).await;
    seed_ship(&pool, 587, "Rifter", 1_067_000.0).await;
    set_ship(&pool, w.owner.character_id, 587).await;
    allow_tracking(&pool, w.map_id, w.owner.user_id).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    wormhole_between(&pool, w.owner, w.map_id, a, b).await;

    // Gate travel is ignored, even without a mapped edge. (587 doubles as the gate's
    // type id; only the FK matters here.)
    sqlx::query(
        "insert into stargates (id, solar_system_id, destination_system_id,
                                destination_stargate_id, type_id)
         values (1, $1, $2, 2, 587)",
    )
    .bind(SYS_A)
    .bind(SYS_C)
    .execute(&pool)
    .await
    .unwrap();
    record_transit(&pool, w.owner.character_id, SYS_A, SYS_C)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar("select count(*) from map_connection_jumps")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);

    // A transit over the mapped hole lands claimed, with the tracked ship's mass.
    record_transit(&pool, w.owner.character_id, SYS_B, SYS_A)
        .await
        .unwrap();
    let row = sqlx::query!(
        r#"select connection_id, character_id, mass, is_manual
           from map_connection_jumps order by id desc limit 1"#
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(row.connection_id.is_some());
    assert_eq!(row.character_id, Some(w.owner.character_id));
    assert_eq!(row.mass, 1_067_000);
    assert!(!row.is_manual);

    // An unmapped hole from a placed origin leaves a pending row, which a connection
    // created shortly after claims.
    record_transit(&pool, w.owner.character_id, SYS_B, SYS_C)
        .await
        .unwrap();
    let pending: i64 =
        sqlx::query_scalar("select count(*) from map_connection_jumps where connection_id is null")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(pending, 1);
    let c = place(&pool, w.owner, w.map_id, SYS_C).await;
    let conn = wormhole_between(&pool, w.owner, w.map_id, b, c).await;
    let claimed: i64 =
        sqlx::query_scalar("select count(*) from map_connection_jumps where connection_id = $1")
            .bind(conn)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(claimed, 1);

    // Without the tracking opt-in nothing is recorded.
    sqlx::query("update map_user_settings set tracking_allowed = false")
        .execute(&pool)
        .await
        .unwrap();
    record_transit(&pool, w.owner.character_id, SYS_A, SYS_B)
        .await
        .unwrap();
    let total: i64 = sqlx::query_scalar("select count(*) from map_connection_jumps")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total, 2);
}

/// One transit is judged map by map: a map where the pilot is only a viewer records
/// nothing, and neither does one that has the pair mapped as a stargate.
#[sqlx::test]
async fn transit_is_recorded_only_on_maps_that_take_it(pool: PgPool) {
    let w = world(&pool).await;
    allow_tracking(&pool, w.map_id, w.owner.user_id).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let hole = wormhole_between(&pool, w.owner, w.map_id, a, b).await;

    // A second map with the same pair as a gate.
    let gates = wormholesystems::maps::map::create_map(
        &pool,
        w.owner,
        &Default::default(),
        wormholesystems::maps::map::CreateMap {
            name: "Gates".into(),
            description: None,
        },
    )
    .await
    .unwrap()
    .id;
    allow_tracking(&pool, gates, w.owner.user_id).await;
    let ga = place(&pool, w.owner, gates, SYS_A).await;
    let gb = place(&pool, w.owner, gates, SYS_B).await;
    add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: gates,
            from_system: ga,
            to_system: gb,
            kind: ConnectionType::Stargate,
            size: None,
        },
    )
    .await
    .unwrap();

    // A third the owner can only view, with the origin placed.
    let other = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;
    let viewed = wormholesystems::maps::map::create_map(
        &pool,
        other,
        &Default::default(),
        wormholesystems::maps::map::CreateMap {
            name: "Theirs".into(),
            description: None,
        },
    )
    .await
    .unwrap()
    .id;
    place(&pool, other, viewed, SYS_A).await;
    wormholesystems::maps::access::set_access(
        &pool,
        other,
        wormholesystems::maps::access::SetAccess {
            map_id: viewed,
            subject_type: wormholesystems::maps::SubjectType::Character,
            subject_id: w.owner.character_id,
            role: Role::Viewer,
            expires_at: None,
        },
    )
    .await
    .unwrap();
    allow_tracking(&pool, viewed, w.owner.user_id).await;

    record_transit(&pool, w.owner.character_id, SYS_A, SYS_B)
        .await
        .unwrap();
    let rows: Vec<(i64, Option<i64>)> =
        sqlx::query_as("select map_id, connection_id from map_connection_jumps")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(rows, [(w.map_id, Some(hole))]);
}

#[sqlx::test]
async fn claim_window_and_prune(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;

    // Two pending rows: one fresh, one outside the claim window.
    for backdate_secs in [10.0_f64, 600.0] {
        sqlx::query(
            "insert into map_connection_jumps
                 (map_id, from_solar_system_id, to_solar_system_id, mass, created_at)
             values ($1, $2, $3, 5, now() - make_interval(secs => $4))",
        )
        .bind(w.map_id)
        .bind(SYS_A)
        .bind(SYS_B)
        .bind(backdate_secs)
        .execute(&pool)
        .await
        .unwrap();
    }

    let conn = wormhole_between(&pool, w.owner, w.map_id, a, b).await;
    let claimed = claim_pending(&pool, w.map_id, conn, SYS_A, SYS_B)
        .await
        .unwrap();
    // add_connection already claimed the fresh row; the stale one stays pending.
    assert_eq!(claimed, 0);
    let on_conn: i64 =
        sqlx::query_scalar("select count(*) from map_connection_jumps where connection_id = $1")
            .bind(conn)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(on_conn, 1);

    // Prune removes only stale pending rows.
    sqlx::query(
        "update map_connection_jumps set created_at = now() - interval '11 minutes'
         where connection_id is null",
    )
    .execute(&pool)
    .await
    .unwrap();
    let pruned = prune_unclaimed(&pool).await.unwrap();
    assert_eq!(pruned, 1);
    let remaining: i64 = sqlx::query_scalar("select count(*) from map_connection_jumps")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining, 1);
}
