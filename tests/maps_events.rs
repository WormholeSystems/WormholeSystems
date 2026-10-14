//! The map history tree: that every mutation records a step, that a step's stored
//! directions actually restore what they describe, and that moving the cursor around
//! settles instead of drifting.

mod common;

use common::{SYS_A, SYS_B, SYS_C, member_with_role, world};
use sqlx::PgPool;
use wormholesystems::maps::connection::{
    AddConnection, RemoveConnection, add_connection, remove_connection,
};
use wormholesystems::maps::connection::{SetConnectionStatus, set_connection_status};
use wormholesystems::maps::events_log::{
    GotoMapEvent, MapEventEntry, MapHistory, MapIdBody, goto, list_history, redo, undo,
};
use wormholesystems::maps::map::{GetMap, get_map};
use wormholesystems::maps::signatures::{
    AddSignature, RemoveSignature, add_signature, list_signatures, remove_signature,
};
use wormholesystems::maps::solar_system::{
    AddSystem, RemoveSystem, RemoveSystems, SetAlias, add_system, remove_system, remove_systems,
    set_alias,
};
use wormholesystems::maps::{
    Actor, ConnectionType, MapConnection, MapError, MapSystemView, MassStatus, Role, SignatureGroup,
};

async fn place(pool: &PgPool, actor: Actor, map_id: i64, sys: i64, x: f64) -> i64 {
    add_system(
        pool,
        actor,
        AddSystem {
            map_id,
            solar_system_id: sys,
            x,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn systems(pool: &PgPool, actor: Actor, map_id: i64) -> Vec<MapSystemView> {
    get_map(pool, actor, GetMap { map_id })
        .await
        .unwrap()
        .systems
}

async fn placed(pool: &PgPool, actor: Actor, map_id: i64) -> Vec<i64> {
    systems(pool, actor, map_id)
        .await
        .iter()
        .filter_map(|s| s.solar_system_id())
        .collect()
}

async fn connections(pool: &PgPool, actor: Actor, map_id: i64) -> Vec<MapConnection> {
    get_map(pool, actor, GetMap { map_id })
        .await
        .unwrap()
        .connections
}

async fn history(pool: &PgPool, actor: Actor, map_id: i64) -> MapHistory {
    list_history(pool, actor, map_id).await.unwrap()
}

/// The newest journal row, which is also the head right after a change.
async fn newest(pool: &PgPool, actor: Actor, map_id: i64) -> MapEventEntry {
    history(pool, actor, map_id).await.entries.remove(0)
}

async fn step_back(pool: &PgPool, actor: Actor, map_id: i64) {
    undo(pool, actor, MapIdBody { map_id }).await.unwrap()
}

async fn step_forward(pool: &PgPool, actor: Actor, map_id: i64) {
    redo(pool, actor, MapIdBody { map_id }).await.unwrap()
}

async fn connect(pool: &PgPool, actor: Actor, map_id: i64, from: i64, to: i64) -> MapConnection {
    add_connection(
        pool,
        actor,
        AddConnection {
            map_id,
            from_system: from,
            to_system: to,
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap()
}

#[sqlx::test]
async fn every_mutation_records_a_step_and_advances_the_cursor(pool: PgPool) {
    let w = world(&pool).await;

    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 100.0).await;
    set_alias(
        &pool,
        w.owner,
        SetAlias {
            map_id: w.map_id,
            map_solar_system_id: a,
            alias: Some("Home".into()),
        },
    )
    .await
    .unwrap();
    connect(&pool, w.owner, w.map_id, a, b).await;

    let h = history(&pool, w.owner, w.map_id).await;
    // Newest first. Creating the map is not itself a map command.
    assert_eq!(h.entries.len(), 4);
    assert_eq!(h.entries[0].kind, "connections.added");
    assert_eq!(h.entries[3].kind, "systems.added");
    // The cursor sits on the newest step, and each step hangs off the one before it.
    assert_eq!(h.head_event_id, Some(h.entries[0].id));
    assert_eq!(h.entries[0].parent_id, Some(h.entries[1].id));
    assert_eq!(h.entries[3].parent_id, None, "the first step is a root");
    for e in &h.entries {
        assert!(!e.label.is_empty(), "every step needs a label");
        assert_eq!(e.character_id, Some(w.owner.character_id));
        assert_eq!(e.character_name.as_deref(), Some("Char 1001"));
        assert!(e.is_step);
        assert!(
            e.applied,
            "nothing has been undone, so every step is in effect"
        );
    }
    assert!(h.can_undo);
    assert!(!h.can_redo);
}

/// The bug this model replaced: undo used to append a new journal row, so after one undo
/// the buttons could only ever offer "redo", which toggled the same change on and off for
/// ever and grew the journal on every press.
/// The listing walks the applied path only as far back as the page reaches, which must
/// not change what it says about the entries on it.
#[sqlx::test]
async fn a_history_longer_than_the_page_still_marks_what_is_applied(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    for i in 0..60 {
        set_alias(
            &pool,
            w.owner,
            SetAlias {
                map_id: w.map_id,
                map_solar_system_id: a,
                alias: Some(format!("A{i}")),
            },
        )
        .await
        .unwrap();
    }
    for _ in 0..5 {
        step_back(&pool, w.owner, w.map_id).await;
    }

    let h = history(&pool, w.owner, w.map_id).await;
    assert_eq!(h.entries.len(), 50, "one page, of 61 steps");
    for (i, e) in h.entries.iter().enumerate() {
        assert_eq!(
            e.applied,
            i >= 5,
            "entry {i}: only the five newest are undone"
        );
    }
    assert_eq!(h.head_event_id, Some(h.entries[5].id));
    assert_eq!(h.undo_target, Some(h.entries[6].id));
    assert_eq!(h.redo_target, Some(h.entries[4].id));
}

#[sqlx::test]
async fn undo_and_redo_settle_instead_of_toggling_for_ever(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let journal_len = history(&pool, w.owner, w.map_id).await.entries.len();

    for _ in 0..3 {
        step_back(&pool, w.owner, w.map_id).await;
        let h = history(&pool, w.owner, w.map_id).await;
        assert!(systems(&pool, w.owner, w.map_id).await.is_empty());
        assert_eq!(h.head_event_id, None, "rewound past the only step");
        assert!(!h.can_undo, "there is nothing further back");
        assert!(h.can_redo);

        step_forward(&pool, w.owner, w.map_id).await;
        let h = history(&pool, w.owner, w.map_id).await;
        assert_eq!(systems(&pool, w.owner, w.map_id).await.len(), 1);
        assert!(h.can_undo);
        assert!(!h.can_redo, "back at the tip, so there is nothing to redo");

        // Walking the cursor never writes a new step.
        assert_eq!(h.entries.len(), journal_len);
    }

    // And undoing at the root is refused rather than doing something surprising.
    step_back(&pool, w.owner, w.map_id).await;
    assert!(matches!(
        undo(&pool, w.owner, MapIdBody { map_id: w.map_id }).await,
        Err(MapError::Conflict(_))
    ));
}

#[sqlx::test]
async fn a_change_after_an_undo_branches_and_the_old_branch_stays_reachable(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    place(&pool, w.owner, w.map_id, SYS_B, 100.0).await;
    let added_b = newest(&pool, w.owner, w.map_id).await.id;

    // Undo B, then do something else: C branches off the same parent instead of erasing B.
    step_back(&pool, w.owner, w.map_id).await;
    place(&pool, w.owner, w.map_id, SYS_C, 200.0).await;
    let added_c = newest(&pool, w.owner, w.map_id).await.id;

    let h = history(&pool, w.owner, w.map_id).await;
    let b = h.entries.iter().find(|e| e.id == added_b).unwrap();
    let c = h.entries.iter().find(|e| e.id == added_c).unwrap();
    assert_eq!(b.parent_id, c.parent_id, "B and C are siblings");
    assert!(!b.applied, "B is on the abandoned branch");
    assert!(c.applied);
    assert_eq!(placed(&pool, w.owner, w.map_id).await, vec![SYS_A, SYS_C]);

    // Redo prefers the newest branch, so it will not silently pull B back.
    assert_eq!(h.redo_target, None, "C is already the tip of its branch");

    // The abandoned branch is still reachable on purpose.
    goto(
        &pool,
        w.owner,
        GotoMapEvent {
            map_id: w.map_id,
            event_id: Some(added_b),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        placed(&pool, w.owner, w.map_id).await,
        vec![SYS_A, SYS_B],
        "C is undone and B is back"
    );
    assert_eq!(
        history(&pool, w.owner, w.map_id).await.head_event_id,
        Some(added_b)
    );
}

#[sqlx::test]
async fn undo_restores_a_removed_system_with_its_edges_and_signatures(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 100.0).await;
    connect(&pool, w.owner, w.map_id, a, b).await;
    add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_B,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Wormhole,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: b,
        },
    )
    .await
    .unwrap();
    assert_eq!(systems(&pool, w.owner, w.map_id).await.len(), 1);
    assert!(
        connections(&pool, w.owner, w.map_id).await.is_empty(),
        "removing a system takes its edges with it"
    );

    assert_eq!(
        newest(&pool, w.owner, w.map_id).await.kind,
        "systems.removed"
    );
    step_back(&pool, w.owner, w.map_id).await;

    let after = systems(&pool, w.owner, w.map_id).await;
    assert_eq!(after.len(), 2);
    // The placement keeps its id, so anything referencing it still resolves.
    assert_eq!(
        after
            .iter()
            .find(|s| s.solar_system_id() == Some(SYS_B))
            .unwrap()
            .id(),
        b
    );
    assert_eq!(
        connections(&pool, w.owner, w.map_id).await.len(),
        1,
        "the cascading edge comes back too"
    );
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(sigs.len(), 1);
    assert_eq!(sigs[0].signature_id, "ABC-123");

    // Redoing removes it again, and the ids survive the round trip so a second undo works.
    step_forward(&pool, w.owner, w.map_id).await;
    assert_eq!(systems(&pool, w.owner, w.map_id).await.len(), 1);
    step_back(&pool, w.owner, w.map_id).await;
    let after = systems(&pool, w.owner, w.map_id).await;
    assert_eq!(after.len(), 2);
    assert_eq!(
        after
            .iter()
            .find(|s| s.solar_system_id() == Some(SYS_B))
            .unwrap()
            .id(),
        b,
        "walking back and forth must not renumber the placement"
    );
}

#[sqlx::test]
async fn undo_round_trips_a_connection_status_change(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 100.0).await;
    let conn = connect(&pool, w.owner, w.map_id, a, b).await;
    assert_eq!(conn.mass_status, None);

    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn.id,
            kind: None,
            mass_status: Some(Some(MassStatus::Critical)),
            time_status: None,
            size: None,
            preserve_mass: None,
        },
    )
    .await
    .unwrap();

    step_back(&pool, w.owner, w.map_id).await;
    // The inverse clears the field rather than leaving it, which is the `Option<Option<_>>`
    // distinction the wire format depends on.
    assert_eq!(
        connections(&pool, w.owner, w.map_id).await[0].mass_status,
        None
    );

    step_forward(&pool, w.owner, w.map_id).await;
    assert_eq!(
        connections(&pool, w.owner, w.map_id).await[0].mass_status,
        Some(MassStatus::Critical)
    );
}

#[sqlx::test]
async fn undo_walks_back_through_several_steps_one_at_a_time(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    set_alias(
        &pool,
        w.owner,
        SetAlias {
            map_id: w.map_id,
            map_solar_system_id: a,
            alias: Some("Staging".into()),
        },
    )
    .await
    .unwrap();

    step_back(&pool, w.owner, w.map_id).await;
    assert_eq!(systems(&pool, w.owner, w.map_id).await[0].alias(), None);

    step_back(&pool, w.owner, w.map_id).await;
    assert!(systems(&pool, w.owner, w.map_id).await.is_empty());

    // Forward again in the same order.
    step_forward(&pool, w.owner, w.map_id).await;
    assert_eq!(systems(&pool, w.owner, w.map_id).await.len(), 1);
    step_forward(&pool, w.owner, w.map_id).await;
    assert_eq!(
        systems(&pool, w.owner, w.map_id).await[0].alias(),
        Some("Staging")
    );
}

/// The history belongs to the map, not to whoever happened to make the change: undo means
/// "step this map back", so a member can reverse a teammate's last action.
#[sqlx::test]
async fn any_member_can_step_the_map_back(pool: PgPool) {
    let w = world(&pool).await;
    let mate = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;
    place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;

    step_back(&pool, mate, w.map_id).await;
    assert!(systems(&pool, w.owner, w.map_id).await.is_empty());
}

#[sqlx::test]
async fn history_is_viewer_readable_and_moving_it_is_member_only(pool: PgPool) {
    let w = world(&pool).await;
    let viewer = member_with_role(&pool, w.owner, w.map_id, 1003, 2003, Role::Viewer).await;
    place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;

    assert_eq!(history(&pool, viewer, w.map_id).await.entries.len(), 1);

    assert!(matches!(
        undo(&pool, viewer, MapIdBody { map_id: w.map_id }).await,
        Err(MapError::Forbidden)
    ));
    assert!(matches!(
        goto(
            &pool,
            viewer,
            GotoMapEvent {
                map_id: w.map_id,
                event_id: None,
            },
        )
        .await,
        Err(MapError::Forbidden)
    ));
}

#[sqlx::test]
async fn a_bulk_removal_undoes_as_one_step(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 100.0).await;
    let c = place(&pool, w.owner, w.map_id, SYS_C, 200.0).await;
    connect(&pool, w.owner, w.map_id, a, b).await;

    remove_systems(
        &pool,
        w.owner,
        RemoveSystems {
            map_id: w.map_id,
            map_solar_system_ids: vec![a, b, c],
        },
    )
    .await
    .unwrap();
    assert!(systems(&pool, w.owner, w.map_id).await.is_empty());
    assert_eq!(
        newest(&pool, w.owner, w.map_id).await.entries_count,
        3,
        "the step counts what it touched"
    );

    step_back(&pool, w.owner, w.map_id).await;
    assert_eq!(systems(&pool, w.owner, w.map_id).await.len(), 3);
    assert_eq!(connections(&pool, w.owner, w.map_id).await.len(), 1);
}

#[sqlx::test]
async fn undoing_a_signature_removal_brings_it_back(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let sig = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "XYZ-999".into(),
            group: SignatureGroup::Data,
            name: Some("Ruined Talocan".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    remove_signature(
        &pool,
        w.owner,
        RemoveSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
        },
    )
    .await
    .unwrap();
    assert!(
        list_signatures(&pool, w.owner, w.map_id)
            .await
            .unwrap()
            .is_empty()
    );

    step_back(&pool, w.owner, w.map_id).await;
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(sigs.len(), 1);
    assert_eq!(sigs[0].signature_id, "XYZ-999");
    assert_eq!(sigs[0].name.as_deref(), Some("Ruined Talocan"));
    assert_eq!(sigs[0].group, SignatureGroup::Data);
}

#[sqlx::test]
async fn removing_a_connection_and_undoing_it_keeps_the_endpoints(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 100.0).await;
    let conn = connect(&pool, w.owner, w.map_id, a, b).await;

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

    step_back(&pool, w.owner, w.map_id).await;
    let conns = connections(&pool, w.owner, w.map_id).await;
    assert_eq!(conns.len(), 1);
    assert_eq!(conns[0].from_system, a);
    assert_eq!(conns[0].to_system, b);
    assert_eq!(conns[0].kind, ConnectionType::Wormhole);
    assert_eq!(systems(&pool, w.owner, w.map_id).await.len(), 2);
}

/// The stale sweep deletes edges *and* the placements they strand, so its undo has to put
/// both back. This is the case a placement-only inverse would get wrong: the C endpoint
/// stays (it is pinned), but the edge into it must come back too.
#[sqlx::test]
async fn cleaning_stale_connections_undoes_as_one_step(pool: PgPool) {
    use wormholesystems::maps::connection::{CleanStaleConnections, clean_stale_connections};
    use wormholesystems::maps::solar_system::{SetPinned, set_pinned};

    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 100.0).await;
    let c = place(&pool, w.owner, w.map_id, SYS_C, 200.0).await;
    set_pinned(
        &pool,
        w.owner,
        SetPinned {
            map_id: w.map_id,
            map_solar_system_id: c,
            value: true,
        },
    )
    .await
    .unwrap();

    for (from, to) in [(a, b), (a, c)] {
        let conn = connect(&pool, w.owner, w.map_id, from, to).await;
        set_connection_status(
            &pool,
            w.owner,
            SetConnectionStatus {
                map_id: w.map_id,
                connection_id: conn.id,
                kind: None,
                mass_status: None,
                time_status: Some(Some(wormholesystems::maps::TimeStatus::Critical)),
                size: None,
                preserve_mass: None,
            },
        )
        .await
        .unwrap();
    }
    sqlx::query(
        "update map_connections set time_status_updated_at = now() - interval '2 hours'
         where map_id = $1",
    )
    .bind(w.map_id)
    .execute(&pool)
    .await
    .unwrap();

    let removed =
        clean_stale_connections(&pool, w.owner, CleanStaleConnections { map_id: w.map_id })
            .await
            .unwrap();
    assert_eq!(removed, 2);
    assert!(connections(&pool, w.owner, w.map_id).await.is_empty());
    let left = systems(&pool, w.owner, w.map_id).await;
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].id(), c);

    assert_eq!(
        newest(&pool, w.owner, w.map_id).await.kind,
        "connections.cleaned"
    );
    step_back(&pool, w.owner, w.map_id).await;
    assert_eq!(systems(&pool, w.owner, w.map_id).await.len(), 3);
    assert_eq!(
        connections(&pool, w.owner, w.map_id).await.len(),
        2,
        "the edge into the pinned system must come back, not just the orphans' edges"
    );

    step_forward(&pool, w.owner, w.map_id).await;
    assert!(connections(&pool, w.owner, w.map_id).await.is_empty());
    assert_eq!(systems(&pool, w.owner, w.map_id).await.len(), 1);
}

/// A fresh critical mark is not stale: sweeping must not touch a hole someone just flagged.
#[sqlx::test]
async fn a_freshly_critical_connection_is_not_swept(pool: PgPool) {
    use wormholesystems::maps::connection::{CleanStaleConnections, clean_stale_connections};

    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 100.0).await;
    let conn = connect(&pool, w.owner, w.map_id, a, b).await;
    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn.id,
            kind: None,
            mass_status: None,
            time_status: Some(Some(wormholesystems::maps::TimeStatus::Critical)),
            size: None,
            preserve_mass: None,
        },
    )
    .await
    .unwrap();

    let err =
        clean_stale_connections(&pool, w.owner, CleanStaleConnections { map_id: w.map_id }).await;
    assert!(matches!(err, Err(MapError::Conflict(_))));
    assert_eq!(connections(&pool, w.owner, w.map_id).await.len(), 1);
}
