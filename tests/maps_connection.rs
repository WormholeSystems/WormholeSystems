//! Connections: connecting and disconnecting placed systems, and marking life-cycle state.

mod common;

use common::catalog::seed_wormhole_system;
use common::map::{connect, connect_kind, place};
use common::{SYS_A, SYS_B, member_with_role, world};
use sqlx::PgPool;
use wormholesystems::maps::connection::{
    AddConnection, RemoveConnection, SetConnectionStatus, add_connection, remove_connection,
    set_connection_status,
};
use wormholesystems::maps::solar_system::{AddSystem, add_system};
use wormholesystems::maps::{ConnectionType, MapError, MassStatus, Role, TimeStatus};

#[sqlx::test]
async fn add_connection_returns_fields_and_validates(pool: PgPool) {
    let w = world(&pool).await;
    let a = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            x: 0.0,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap();
    let b = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
            x: 1.0,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap();

    // Self-connection rejected (pure validation).
    assert!(matches!(
        add_connection(
            &pool,
            w.owner,
            AddConnection {
                map_id: w.map_id,
                from_system: a.id,
                to_system: a.id,
                kind: ConnectionType::Wormhole,
                size: None,
            }
        )
        .await,
        Err(MapError::Validation(_)),
    ));
    // Endpoint not on the map rejected.
    assert!(matches!(
        add_connection(
            &pool,
            w.owner,
            AddConnection {
                map_id: w.map_id,
                from_system: a.id,
                to_system: 4242,
                kind: ConnectionType::Wormhole,
                size: None,
            }
        )
        .await,
        Err(MapError::Validation(_)),
    ));

    let conn = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: a.id,
            to_system: b.id,
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(conn.from_system, a.id);
    assert_eq!(conn.to_system, b.id);
    assert_eq!(conn.kind, ConnectionType::Wormhole);

    // The same pair may be connected again (parallel edges are allowed): a second,
    // distinct connection is created — even reversed and of a different kind.
    let dup = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: b.id,
            to_system: a.id,
            kind: ConnectionType::Stargate,
            size: None,
        },
    )
    .await
    .unwrap();
    assert_ne!(dup.id, conn.id);
    let edges: i64 = sqlx::query_scalar("select count(*) from map_connections where map_id = $1")
        .bind(w.map_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(edges, 2);
}

#[sqlx::test]
async fn remove_connection_clears_signature_link(pool: PgPool) {
    let w = world(&pool).await;
    let a = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            x: 0.0,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap();
    let b = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
            x: 1.0,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap();
    let conn = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: a.id,
            to_system: b.id,
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();

    sqlx::query(
        r#"insert into signatures (map_id, solar_system_id, signature_id, "group", connection_id)
           values ($1, $2, 'ABC-123', 'wormhole', $3)"#,
    )
    .bind(w.map_id)
    .bind(SYS_A)
    .bind(conn.id)
    .execute(&pool)
    .await
    .unwrap();

    remove_connection(
        &pool,
        w.owner,
        RemoveConnection {
            map_id: w.map_id,
            connection_id: conn.id,
        },
    )
    .await
    .unwrap();

    // The signature survives, with its connection_id cleared.
    let total: i64 = sqlx::query_scalar("select count(*) from signatures where map_id = $1")
        .bind(w.map_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let linked: i64 =
        sqlx::query_scalar("select count(*) from signatures where connection_id is not null")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(total, 1);
    assert_eq!(linked, 0);

    // Unknown connection → NotFound.
    assert!(matches!(
        remove_connection(
            &pool,
            w.owner,
            RemoveConnection {
                map_id: w.map_id,
                connection_id: 4242
            }
        )
        .await,
        Err(MapError::NotFound),
    ));
}

#[sqlx::test]
async fn set_status_marks_a_connection_without_a_signature(pool: PgPool) {
    let w = world(&pool).await;
    let a = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            x: 0.0,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap();
    let b = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
            x: 1.0,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap();
    let conn = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: a.id,
            to_system: b.id,
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(
        conn.mass_status, None,
        "new connections start with unknown state"
    );

    let viewer = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Viewer).await;
    assert!(matches!(
        set_connection_status(
            &pool,
            viewer,
            SetConnectionStatus {
                map_id: w.map_id,
                connection_id: conn.id,
                mass_status: Some(Some(MassStatus::Critical)),
                ..Default::default()
            }
        )
        .await,
        Err(MapError::Forbidden),
    ));

    // Mark mass + EOL with no signature linked.
    let marked = set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn.id,
            mass_status: Some(Some(MassStatus::Critical)),
            time_status: Some(Some(TimeStatus::Eol)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(marked.mass_status, Some(MassStatus::Critical));
    assert_eq!(marked.time_status, Some(TimeStatus::Eol));

    // Partial: clear only the time, leave mass untouched.
    let cleared = set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn.id,
            time_status: Some(None),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        cleared.mass_status,
        Some(MassStatus::Critical),
        "mass left unchanged"
    );
    assert_eq!(cleared.time_status, None, "time cleared");

    // Unknown connection → NotFound.
    assert!(matches!(
        set_connection_status(
            &pool,
            w.owner,
            SetConnectionStatus {
                map_id: w.map_id,
                connection_id: 4242,
                mass_status: Some(Some(MassStatus::Stable)),
                ..Default::default()
            }
        )
        .await,
        Err(MapError::NotFound),
    ));
}

async fn age_by(pool: &PgPool, connection_id: i64, hours: i32) {
    sqlx::query(
        "update map_connections set created_at = now() - make_interval(hours => $2) where id = $1",
    )
    .bind(connection_id)
    .bind(hours)
    .execute(pool)
    .await
    .unwrap();
}

async fn marked_ago(pool: &PgPool, connection_id: i64, hours: i32) {
    sqlx::query(
        "update map_connections set time_status_updated_at = now() - make_interval(hours => $2)
         where id = $1",
    )
    .bind(connection_id)
    .bind(hours)
    .execute(pool)
    .await
    .unwrap();
}

/// A wormhole signature in Jita, already linked to the connection.
async fn link_signature_row(pool: &PgPool, map_id: i64, connection_id: i64) -> i64 {
    sqlx::query_scalar(
        r#"insert into signatures (map_id, solar_system_id, signature_id, "group", connection_id)
           values ($1, $2, 'ABC-123', 'wormhole', $3) returning id"#,
    )
    .bind(map_id)
    .bind(SYS_A)
    .bind(connection_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn status_of(pool: &PgPool, connection_id: i64) -> Option<TimeStatus> {
    sqlx::query_scalar("select time_status from map_connections where id = $1")
        .bind(connection_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// The clock marks a plain hole EOL at 20 hours and critical at 23, records each mark as a
/// background audit entry, and the marks reach a linked signature through the sync.
#[sqlx::test]
async fn ageing_marks_a_hole_eol_then_critical(pool: PgPool) {
    use wormholesystems::maps::connection::age_connections;
    use wormholesystems::maps::events_log::list_history;

    let w = world(&pool).await;
    seed_wormhole_system(&pool, 31000001, 3).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let j = place(&pool, w.owner, w.map_id, 31000001).await;
    let conn = connect(&pool, w.owner, w.map_id, a, j).await.id;
    let sig = link_signature_row(&pool, w.map_id, conn).await;

    age_by(&pool, conn, 19).await;
    assert_eq!(age_connections(&pool).await.unwrap(), 0);
    assert_eq!(
        status_of(&pool, conn).await,
        None,
        "a young hole stays unknown"
    );

    age_by(&pool, conn, 20).await;
    assert_eq!(age_connections(&pool).await.unwrap(), 1);
    assert_eq!(status_of(&pool, conn).await, Some(TimeStatus::Eol));
    assert_eq!(
        age_connections(&pool).await.unwrap(),
        0,
        "a second pass finds nothing new to mark"
    );

    let newest = list_history(&pool, w.owner, w.map_id)
        .await
        .unwrap()
        .entries
        .remove(0);
    assert_eq!(newest.kind, "connections.aged");
    assert_eq!(newest.character_id, None, "the clock is not a pilot");
    assert!(!newest.is_step, "a background mark is not an undo step");

    let linked: Option<TimeStatus> =
        sqlx::query_scalar("select time_status from signatures where id = $1")
            .bind(sig)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        linked,
        Some(TimeStatus::Eol),
        "the sync carries the mark to the signature"
    );

    // Once EOL, the hole is judged from its mark: critical three hours after it.
    age_by(&pool, conn, 30).await;
    marked_ago(&pool, conn, 2).await;
    assert_eq!(age_connections(&pool).await.unwrap(), 0);
    marked_ago(&pool, conn, 3).await;
    assert_eq!(age_connections(&pool).await.unwrap(), 1);
    assert_eq!(status_of(&pool, conn).await, Some(TimeStatus::Critical));
}

/// C6 and drifter holes into known space run on their own clocks; a hole a pilot already
/// marked, and a stargate, are left alone.
#[sqlx::test]
async fn ageing_uses_the_class_pair_and_never_downgrades(pool: PgPool) {
    use wormholesystems::maps::connection::age_connections;

    let w = world(&pool).await;
    sqlx::query("update solar_systems set wormhole_class_id = 7 where id = $1")
        .bind(SYS_A)
        .execute(&pool)
        .await
        .unwrap();
    seed_wormhole_system(&pool, 31000006, 6).await;
    seed_wormhole_system(&pool, 31000015, 15).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let c6 = place(&pool, w.owner, w.map_id, 31000006).await;
    let drifter = place(&pool, w.owner, w.map_id, 31000015).await;

    let to_c6 = connect(&pool, w.owner, w.map_id, a, c6).await.id;
    let to_drifter = connect(&pool, w.owner, w.map_id, drifter, a).await.id;
    let gate = connect_kind(&pool, w.owner, w.map_id, a, b, ConnectionType::Stargate)
        .await
        .id;
    let marked = connect(&pool, w.owner, w.map_id, b, c6).await.id;
    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: marked,
            time_status: Some(Some(TimeStatus::Critical)),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    for id in [to_c6, to_drifter, gate, marked] {
        age_by(&pool, id, 30).await;
    }
    assert_eq!(age_connections(&pool).await.unwrap(), 1);
    assert_eq!(
        status_of(&pool, to_c6).await,
        None,
        "a C6 hole has 48 hours"
    );
    assert_eq!(
        status_of(&pool, to_drifter).await,
        Some(TimeStatus::Critical)
    );
    assert_eq!(status_of(&pool, gate).await, None, "stargates do not age");
    assert_eq!(status_of(&pool, marked).await, Some(TimeStatus::Critical));

    age_by(&pool, to_c6, 44).await;
    assert_eq!(age_connections(&pool).await.unwrap(), 1);
    assert_eq!(status_of(&pool, to_c6).await, Some(TimeStatus::Eol));
}

/// Expired is only ever read off the hole: the clock leaves an expired hole where it is,
/// and the stale cleanup takes it an hour after the mark, like a critical one.
#[sqlx::test]
async fn an_expired_hole_is_not_aged_and_goes_stale_after_an_hour(pool: PgPool) {
    use wormholesystems::maps::connection::{
        CleanStaleConnections, age_connections, clean_stale_connections, list_stale_connections,
    };

    let w = world(&pool).await;
    seed_wormhole_system(&pool, 31000001, 3).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let j = place(&pool, w.owner, w.map_id, 31000001).await;
    let conn = connect(&pool, w.owner, w.map_id, a, j).await.id;
    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn,
            time_status: Some(Some(TimeStatus::Expired)),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    age_by(&pool, conn, 30).await;
    assert_eq!(age_connections(&pool).await.unwrap(), 0);
    assert_eq!(status_of(&pool, conn).await, Some(TimeStatus::Expired));

    assert!(
        list_stale_connections(&pool, w.owner, w.map_id)
            .await
            .unwrap()
            .is_empty(),
        "a fresh mark is not stale yet"
    );
    marked_ago(&pool, conn, 1).await;
    let stale = list_stale_connections(&pool, w.owner, w.map_id)
        .await
        .unwrap();
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].connection_id, conn);

    let removed =
        clean_stale_connections(&pool, w.owner, CleanStaleConnections { map_id: w.map_id })
            .await
            .unwrap();
    assert_eq!(removed, 1);
}

/// Worst wins on link: an expired signature drags a critical connection along.
#[sqlx::test]
async fn an_expired_signature_wins_over_a_critical_connection(pool: PgPool) {
    let w = world(&pool).await;
    seed_wormhole_system(&pool, 31000001, 3).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let j = place(&pool, w.owner, w.map_id, 31000001).await;
    let conn = connect(&pool, w.owner, w.map_id, a, j).await.id;
    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn,
            time_status: Some(Some(TimeStatus::Critical)),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let sig: i64 = sqlx::query_scalar(
        r#"insert into signatures (map_id, solar_system_id, signature_id, "group", time_status)
           values ($1, $2, 'ABC-123', 'wormhole', 'expired') returning id"#,
    )
    .bind(w.map_id)
    .bind(SYS_A)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("update signatures set connection_id = $2 where id = $1")
        .bind(sig)
        .bind(conn)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(status_of(&pool, conn).await, Some(TimeStatus::Expired));
}

/// A wormhole three days old cannot exist: it goes, with the placement it strands, while
/// pinned systems, stargates and younger holes stay. Its signature is unlinked, not deleted.
#[sqlx::test]
async fn expiry_removes_dead_wormholes_and_what_they_strand(pool: PgPool) {
    use wormholesystems::maps::connection::expire_connections;
    use wormholesystems::maps::events_log::list_history;
    use wormholesystems::maps::solar_system::{SetPinned, set_pinned};

    let w = world(&pool).await;
    seed_wormhole_system(&pool, 31000001, 3).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let c = place(&pool, w.owner, w.map_id, common::SYS_C).await;
    let j = place(&pool, w.owner, w.map_id, 31000001).await;
    set_pinned(
        &pool,
        w.owner,
        SetPinned {
            map_id: w.map_id,
            map_solar_system_id: a,
            value: true,
        },
    )
    .await
    .unwrap();

    let dead = connect(&pool, w.owner, w.map_id, a, j).await.id;
    let gate = connect_kind(&pool, w.owner, w.map_id, a, b, ConnectionType::Stargate)
        .await
        .id;
    let young = connect(&pool, w.owner, w.map_id, b, c).await.id;
    let sig = link_signature_row(&pool, w.map_id, dead).await;
    for id in [dead, gate] {
        age_by(&pool, id, 3 * 24 + 1).await;
    }
    age_by(&pool, young, 3 * 24 - 1).await;

    assert_eq!(expire_connections(&pool).await.unwrap(), 1);
    assert_eq!(expire_connections(&pool).await.unwrap(), 0);

    let left: Vec<i64> =
        sqlx::query_scalar("select id from map_connections where map_id = $1 order by id")
            .bind(w.map_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(left, vec![gate, young]);
    let placed: Vec<i64> =
        sqlx::query_scalar("select id from map_solar_systems where map_id = $1 order by id")
            .bind(w.map_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        placed,
        vec![a, b, c],
        "the stranded J-system goes, the pinned one stays"
    );

    let link: Option<i64> =
        sqlx::query_scalar("select connection_id from signatures where id = $1")
            .bind(sig)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(link, None);

    let newest = list_history(&pool, w.owner, w.map_id)
        .await
        .unwrap()
        .entries
        .remove(0);
    assert_eq!(newest.kind, "connections.expired");
    assert_eq!(newest.character_id, None);
}

/// When a row was made and when its lifetime was last marked.
async fn clocks_of(
    pool: &PgPool,
    table: &str,
    id: i64,
) -> (
    chrono::DateTime<chrono::Utc>,
    Option<chrono::DateTime<chrono::Utc>>,
) {
    let sql = match table {
        "map_connections" => {
            "select created_at, time_status_updated_at from map_connections where id = $1"
        }
        _ => "select created_at, time_status_updated_at from signatures where id = $1",
    };
    sqlx::query_as(sql).bind(id).fetch_one(pool).await.unwrap()
}

/// Undoing a removal puts the hole and the scan back as old as they were and EOL since when
/// they were, so the ageing and the three-day expiry carry on rather than start over.
#[sqlx::test]
async fn an_undone_removal_keeps_its_clocks(pool: PgPool) {
    use wormholesystems::maps::events_log::{MapIdBody, undo};
    use wormholesystems::maps::signatures::{RemoveSignature, remove_signature};

    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let conn = connect(&pool, w.owner, w.map_id, a, b).await.id;
    sqlx::query("update map_connections set time_status = 'eol' where id = $1")
        .bind(conn)
        .execute(&pool)
        .await
        .unwrap();
    age_by(&pool, conn, 30).await;
    marked_ago(&pool, conn, 5).await;
    let sig: i64 = sqlx::query_scalar(
        r#"insert into signatures
               (map_id, solar_system_id, signature_id, "group", time_status, created_at,
                time_status_updated_at)
           values ($1, $2, 'XYZ-987', 'wormhole', 'eol', now() - interval '2 days',
                   now() - interval '3 hours')
           returning id"#,
    )
    .bind(w.map_id)
    .bind(SYS_B)
    .fetch_one(&pool)
    .await
    .unwrap();
    let connection_clocks = clocks_of(&pool, "map_connections", conn).await;
    let signature_clocks = clocks_of(&pool, "signatures", sig).await;

    remove_connection(
        &pool,
        w.owner,
        RemoveConnection {
            map_id: w.map_id,
            connection_id: conn,
        },
    )
    .await
    .unwrap();
    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(
        clocks_of(&pool, "map_connections", conn).await,
        connection_clocks
    );

    remove_signature(
        &pool,
        w.owner,
        RemoveSignature {
            map_id: w.map_id,
            signature_pk: sig,
        },
    )
    .await
    .unwrap();
    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(clocks_of(&pool, "signatures", sig).await, signature_clocks);
}
