//! Putting EVE Scout's public holes on the map: what one run places, that a second run
//! changes nothing, and that the whole thing comes back out as one step.

mod common;

use common::catalog::{C247, THERA, seed_catalog, seed_thera};
use common::{SYS_A, SYS_B, SYS_C, member_with_role, world};
use sqlx::PgPool;
use wormholesystems::maps::eve_scout::{
    AddEveScoutConnections, EveScoutHole, add_eve_scout_connections,
};
use wormholesystems::maps::events_log::{MapIdBody, list_history, redo, undo};
use wormholesystems::maps::map::{GetMap, get_map};
use wormholesystems::maps::signatures::{AddSignature, add_signature, list_signatures};
use wormholesystems::maps::solar_system::{AddSystem, add_system};
use wormholesystems::maps::{
    Actor, MapError, MassStatus, Role, SignatureGroup, TimeStatus, WormholeSize,
};

fn hole(solar_system_id: i64, hub_signature: &str, signature: &str) -> EveScoutHole {
    EveScoutHole {
        solar_system_id,
        hub_signature: hub_signature.into(),
        signature: signature.into(),
        wormhole_type: Some("C247".into()),
        mass_status: MassStatus::Reduced,
        time_status: TimeStatus::Eol,
        size: Some(WormholeSize::Large),
    }
}

fn thera_holes(map_id: i64, holes: Vec<EveScoutHole>) -> AddEveScoutConnections {
    AddEveScoutConnections {
        map_id,
        hub_solar_system_id: THERA,
        at: None,
        holes,
    }
}

async fn view(pool: &PgPool, actor: Actor, map_id: i64) -> wormholesystems::maps::MapView {
    get_map(pool, actor, GetMap { map_id }).await.unwrap()
}

#[sqlx::test]
async fn puts_the_hub_its_holes_and_both_ends_signatures_on_the_map(pool: PgPool) {
    let w = world(&pool).await;
    seed_catalog(&pool).await;
    seed_thera(&pool).await;

    let added = add_eve_scout_connections(
        &pool,
        w.owner,
        thera_holes(
            w.map_id,
            vec![
                hole(SYS_A, "THA-001", "FAR-001"),
                hole(SYS_B, "THA-002", ""),
            ],
        ),
    )
    .await
    .unwrap();
    assert_eq!(added, 2);

    let v = view(&pool, w.owner, w.map_id).await;
    let mut placed: Vec<i64> = v
        .systems
        .iter()
        .filter_map(|s| s.solar_system_id())
        .collect();
    placed.sort_unstable();
    assert_eq!(placed, vec![SYS_A, SYS_B, THERA]);
    assert_eq!(v.connections.len(), 2);
    for c in &v.connections {
        assert_eq!(c.mass_status, Some(MassStatus::Reduced));
        assert_eq!(c.time_status, Some(TimeStatus::Eol));
        // C247 admits a cruiser, whatever EVE Scout's own word for the hole was.
        assert_eq!(c.size, Some(WormholeSize::Medium));
    }

    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    let in_thera: Vec<&str> = sigs
        .iter()
        .filter(|s| s.solar_system_id == THERA)
        .map(|s| s.signature_id.as_str())
        .collect();
    assert_eq!(in_thera.len(), 2);
    assert!(in_thera.contains(&"THA-001") && in_thera.contains(&"THA-002"));
    let far = sigs.iter().find(|s| s.solar_system_id == SYS_A).unwrap();
    assert_eq!(far.signature_id, "FAR-001");
    assert_eq!(far.group, SignatureGroup::Wormhole);
    assert_eq!(far.signature_type_id, Some(C247));
    assert!(far.connection_id.is_some());
    // The far side EVE Scout has not scanned yet gets no made-up signature.
    assert!(!sigs.iter().any(|s| s.solar_system_id == SYS_B));
}

#[sqlx::test]
async fn a_second_run_reuses_what_is_there_and_adds_nothing(pool: PgPool) {
    let w = world(&pool).await;
    seed_catalog(&pool).await;
    seed_thera(&pool).await;
    let placed_before = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            x: 1000.0,
            y: 500.0,
            alias: None,
        },
    )
    .await
    .unwrap();

    let holes = vec![hole(SYS_A, "THA-001", "FAR-001")];
    let first = add_eve_scout_connections(&pool, w.owner, thera_holes(w.map_id, holes.clone()))
        .await
        .unwrap();
    let second = add_eve_scout_connections(&pool, w.owner, thera_holes(w.map_id, holes))
        .await
        .unwrap();
    assert_eq!((first, second), (1, 0));

    let v = view(&pool, w.owner, w.map_id).await;
    assert_eq!(v.systems.len(), 2);
    assert_eq!(v.connections.len(), 1);
    let kept = v
        .systems
        .iter()
        .find(|s| s.id() == placed_before.id)
        .unwrap();
    assert_eq!(
        kept.position(),
        (1000.0, 500.0),
        "a placed system stays put"
    );
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(sigs.len(), 2);

    // A run that changed nothing leaves nothing to undo.
    let history = list_history(&pool, w.owner, w.map_id).await.unwrap();
    assert!(!history.entries[0].is_step);
}

#[sqlx::test]
async fn an_unknown_scan_on_either_end_becomes_the_hole(pool: PgPool) {
    let w = world(&pool).await;
    seed_catalog(&pool).await;
    seed_thera(&pool).await;
    add_system(
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
    let scanned = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "FAR-001".into(),
            group: SignatureGroup::Unknown,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    add_eve_scout_connections(
        &pool,
        w.owner,
        thera_holes(w.map_id, vec![hole(SYS_A, "THA-001", "FAR-001")]),
    )
    .await
    .unwrap();

    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    let promoted = sigs.iter().find(|s| s.id == scanned.id).unwrap();
    assert_eq!(promoted.group, SignatureGroup::Wormhole);
    assert_eq!(promoted.signature_type_id, Some(C247));
    assert!(promoted.connection_id.is_some());
    assert_eq!(sigs.len(), 2, "the scan was reused, not duplicated");
}

#[sqlx::test]
async fn the_whole_run_is_one_undo(pool: PgPool) {
    let w = world(&pool).await;
    seed_catalog(&pool).await;
    seed_thera(&pool).await;
    add_system(
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
    let scanned = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "FAR-001".into(),
            group: SignatureGroup::Unknown,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    add_eve_scout_connections(
        &pool,
        w.owner,
        thera_holes(
            w.map_id,
            vec![
                hole(SYS_A, "THA-001", "FAR-001"),
                hole(SYS_C, "THA-003", "FAR-003"),
            ],
        ),
    )
    .await
    .unwrap();
    let history = list_history(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(history.entries[0].kind, "evescout.added");

    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    let v = view(&pool, w.owner, w.map_id).await;
    assert_eq!(
        v.systems.len(),
        1,
        "Thera and the far side it brought are gone"
    );
    assert_eq!(v.connections.len(), 0);
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(sigs.len(), 1, "the signatures the run added went with it");
    let restored = sigs.iter().find(|s| s.id == scanned.id).unwrap();
    assert_eq!(restored.group, SignatureGroup::Unknown);
    assert_eq!(restored.signature_type_id, None);
    assert_eq!(restored.connection_id, None);

    redo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    let v = view(&pool, w.owner, w.map_id).await;
    assert_eq!(v.systems.len(), 3);
    assert_eq!(v.connections.len(), 2);
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(sigs.len(), 4);
}

#[sqlx::test]
async fn only_thera_and_turnur_are_hubs(pool: PgPool) {
    let w = world(&pool).await;
    seed_catalog(&pool).await;
    seed_thera(&pool).await;
    let err = add_eve_scout_connections(
        &pool,
        w.owner,
        AddEveScoutConnections {
            hub_solar_system_id: SYS_A,
            ..thera_holes(w.map_id, vec![hole(SYS_B, "AAA-001", "BBB-001")])
        },
    )
    .await;
    assert!(matches!(err, Err(MapError::Validation(_))));
}

#[sqlx::test]
async fn a_viewer_may_not(pool: PgPool) {
    let w = world(&pool).await;
    seed_catalog(&pool).await;
    seed_thera(&pool).await;
    let viewer = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Viewer).await;
    let err = add_eve_scout_connections(
        &pool,
        viewer,
        thera_holes(w.map_id, vec![hole(SYS_A, "THA-001", "FAR-001")]),
    )
    .await;
    assert!(matches!(err, Err(MapError::Forbidden)));
}
