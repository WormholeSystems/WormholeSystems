//! Ghost placements: the far side of a wormhole before anyone has been through it, and
//! what happens when someone finally says what it is.

mod common;

use common::map::{connect, place};
use common::{SYS_A, SYS_B, SYS_C, world};
use sqlx::PgPool;
use wormholesystems::maps::MapSystemView;
use wormholesystems::maps::connection::{RemoveConnection, remove_connection};
use wormholesystems::maps::ghost::{ResolveGhostSystem, resolve_ghost_system};
use wormholesystems::maps::history::{MapIdBody, list_history, redo, undo};
use wormholesystems::maps::map::{GetMap, get_map};
use wormholesystems::maps::map::{UpdateMap, update_map};
use wormholesystems::maps::signatures::{
    AddSignature, LinkSignature, PasteSignatures, PastedSignature, RemoveSignature,
    UnlinkSignature, UpdateSignature, add_signature, link_signature, list_signatures,
    paste_signatures, remove_signature, unlink_signature, update_signature,
};
use wormholesystems::maps::solar_system::{
    RemoveSystem, RemoveSystems, SetAlias, SetPinned, remove_system, remove_systems, set_alias,
    set_pinned,
};
use wormholesystems::maps::{
    Actor, MapError, MassStatus, SignatureGroup, TimeStatus, WormholeSize,
};
use wormholesystems::realtime::MapEvent;

async fn scan(pool: &PgPool, actor: Actor, map_id: i64, system: i64, sig: &str) -> i64 {
    add_signature(
        pool,
        actor,
        AddSignature {
            map_id,
            solar_system_id: system,
            signature_id: sig.into(),
            group: SignatureGroup::Wormhole,
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .id
}

/// A map that draws the far side of every hole it knows about. Ghosts are never placed by
/// hand here, any more than they are in the app: the map raises them, and these tests say
/// so by scanning.
async fn ghosting(pool: &PgPool, actor: Actor, map_id: i64) {
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
}

/// The node the map raised for this scan.
async fn ghost_for(pool: &PgPool, map_id: i64, sig: i64) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "select case when f.solar_system_id is null then f.id else t.id end
         from signatures s
         join map_connections c on c.id = s.connection_id
         join map_solar_systems f on f.id = c.from_system
         join map_solar_systems t on t.id = c.to_system
         where s.id = $1 and s.map_id = $2",
    )
    .bind(sig)
    .bind(map_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Where a node sits, for the assertions that care that it stayed there.
async fn position(pool: &PgPool, id: i64) -> (f64, f64) {
    sqlx::query_as::<_, (f64, f64)>(
        "select position_x, position_y from map_solar_systems where id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test]
async fn a_ghost_is_a_placement_with_no_system_hanging_off_the_scan(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

    // It is on the map like any other node, with an edge from where it was scanned.
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    // Nothing to name it after yet, and now the type says so rather than a row of nulls:
    // a ghost carries no name, region or statics to be read off it at all.
    let placed = view.systems.iter().find(|s| s.id() == ghost).unwrap();
    assert!(matches!(placed, MapSystemView::Ghost { .. }));
    assert_eq!(view.connections.len(), 1);
    assert_eq!(view.connections[0].from_system, home);
    assert_eq!(view.connections[0].to_system, ghost);

    // And the signature it came from is linked to that edge, so the two stay in step.
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(sigs[0].connection_id, Some(view.connections[0].id));

    // The map settles its ghosts after every write, so an unrelated one does not raise a
    // second node for a hole that already has one.
    set_alias(
        &pool,
        w.owner,
        SetAlias {
            map_id: w.map_id,
            map_solar_system_id: home,
            alias: Some("Home".into()),
        },
    )
    .await
    .unwrap();
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 2);
}

#[sqlx::test]
async fn resolving_names_the_ghost_and_keeps_where_it_sits(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;
    let where_it_sat = position(&pool, ghost).await;

    let resolved = resolve_ghost_system(
        &pool,
        w.owner,
        ResolveGhostSystem {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            solar_system_id: Some(SYS_B),
            alias: Some("1a".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(resolved.id, ghost, "the node keeps its identity");
    assert_eq!(resolved.solar_system_id, Some(SYS_B));
    assert_eq!(
        (resolved.position_x, resolved.position_y),
        where_it_sat,
        "and where it was drawn"
    );
    assert_eq!(resolved.alias.as_deref(), Some("1a"));

    // Only ghosts are resolvable; a real system is not re-pointed at another one.
    assert!(matches!(
        resolve_ghost_system(
            &pool,
            w.owner,
            ResolveGhostSystem {
                map_id: w.map_id,
                map_solar_system_id: ghost,
                solar_system_id: Some(SYS_C),
                ..Default::default()
            }
        )
        .await,
        Err(MapError::Validation(_)),
    ));
}

#[sqlx::test]
async fn a_ghost_that_leads_back_into_the_chain_is_merged(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    let far = place(&pool, w.owner, w.map_id, SYS_C).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

    // The hole turns out to lead to a system already on the map, from the other side.
    let resolved = resolve_ghost_system(
        &pool,
        w.owner,
        ResolveGhostSystem {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            solar_system_id: Some(SYS_C),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(resolved.id, far, "merged into the placement already there");

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 2, "the ghost is gone, not duplicated");
    assert!(view.systems.iter().all(|s| s.id() != ghost));
    assert_eq!(view.connections.len(), 1);
    assert_eq!(
        (
            view.connections[0].from_system,
            view.connections[0].to_system
        ),
        (home, far),
        "the edge moved onto the real system"
    );
}

#[sqlx::test]
async fn a_ghost_cannot_lead_to_the_system_it_hangs_off(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

    assert!(matches!(
        resolve_ghost_system(
            &pool,
            w.owner,
            ResolveGhostSystem {
                map_id: w.map_id,
                map_solar_system_id: ghost,
                solar_system_id: Some(SYS_A),
                ..Default::default()
            }
        )
        .await,
        Err(MapError::Conflict(_)),
    ));
}

#[sqlx::test]
async fn undoing_a_merge_puts_the_ghost_back_with_its_edge(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    place(&pool, w.owner, w.map_id, SYS_C).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;
    set_alias(
        &pool,
        w.owner,
        SetAlias {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            alias: Some("1a".into()),
        },
    )
    .await
    .unwrap();
    resolve_ghost_system(
        &pool,
        w.owner,
        ResolveGhostSystem {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            solar_system_id: Some(SYS_C),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    let back = view.systems.iter().find(|s| s.id() == ghost).unwrap();
    assert_eq!(back.solar_system_id(), None);
    assert_eq!(back.alias(), Some("1a"));
    assert_eq!(view.connections.len(), 1);
    assert_eq!(
        (
            view.connections[0].from_system,
            view.connections[0].to_system
        ),
        (home, ghost),
        "the edge went back to the ghost"
    );
}

#[sqlx::test]
async fn undoing_a_resolve_makes_it_a_ghost_again(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;
    resolve_ghost_system(
        &pool,
        w.owner,
        ResolveGhostSystem {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            solar_system_id: Some(SYS_B),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    let back = view.systems.iter().find(|s| s.id() == ghost).unwrap();
    assert_eq!(back.solar_system_id(), None);
}

/// A pin set since the resolve by a write that is not a step of its own, an import say,
/// sits on top of the undo stack without being in it. Stepping back over the resolve still
/// works: a ghost cannot be pinned, so the pin goes the way home and rally already do.
#[sqlx::test]
async fn undoing_a_resolve_drops_a_pin_set_since(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;
    resolve_ghost_system(
        &pool,
        w.owner,
        ResolveGhostSystem {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            solar_system_id: Some(SYS_B),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    sqlx::query("update map_solar_systems set is_pinned = true where id = $1")
        .bind(ghost)
        .execute(&pool)
        .await
        .unwrap();

    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    let back = view.systems.iter().find(|s| s.id() == ghost).unwrap();
    assert!(matches!(
        back,
        MapSystemView::Ghost {
            is_pinned: false,
            ..
        }
    ));
}

#[sqlx::test]
async fn a_pasted_scan_raises_the_holes_it_found(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    // Off by default: a scan changes nothing but the signature list.
    paste_signatures(
        &pool,
        w.owner,
        PasteSignatures {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signatures: vec![PastedSignature {
                signature_id: "ABC-123".into(),
                group: Some(SignatureGroup::Wormhole),
                signature_type_id: None,
                name: None,
            }],
        },
    )
    .await
    .unwrap();
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1);

    sqlx::query("update maps set ghost_unlinked_wormholes = true where id = $1")
        .bind(w.map_id)
        .execute(&pool)
        .await
        .unwrap();

    // Two more holes, and the one already scanned: every unmapped hole gets a node, and
    // the whole thing is one entry in the history.
    paste_signatures(
        &pool,
        w.owner,
        PasteSignatures {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signatures: vec![
                PastedSignature {
                    signature_id: "DEF-456".into(),
                    group: Some(SignatureGroup::Wormhole),
                    signature_type_id: None,
                    name: None,
                },
                PastedSignature {
                    signature_id: "GHI-789".into(),
                    group: Some(SignatureGroup::Wormhole),
                    signature_type_id: None,
                    name: None,
                },
            ],
        },
    )
    .await
    .unwrap();

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    let ghosts: Vec<_> = view
        .systems
        .iter()
        .filter(|s| s.solar_system_id().is_none())
        .collect();
    assert_eq!(
        ghosts.len(),
        3,
        "one per unmapped hole, the older one included"
    );
    assert_eq!(view.connections.len(), 3);
    assert!(view.connections.iter().all(|c| c.from_system == home));
    // Siblings stack in the column beside the system they hang off.
    assert!(
        ghosts
            .iter()
            .all(|g| g.position().0 == ghosts[0].position().0)
    );
    let mut ys: Vec<i64> = ghosts.iter().map(|g| g.position().1 as i64).collect();
    ys.sort_unstable();
    ys.dedup();
    assert_eq!(ys.len(), 3, "and do not sit on top of each other");

    // Every signature is linked to the node it raised.
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert!(sigs.iter().all(|s| s.connection_id.is_some()));

    // One undo takes the scan and its nodes back together.
    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1, "the paste's nodes went with it");
    assert!(view.connections.is_empty());
}

#[sqlx::test]
async fn calling_a_signature_a_wormhole_raises_one_too(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    sqlx::query("update maps set ghost_unlinked_wormholes = true where id = $1")
        .bind(w.map_id)
        .execute(&pool)
        .await
        .unwrap();

    // Typed in by hand and uncategorised: nothing is known about it, so nothing is drawn.
    let sig = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Unknown,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1);

    update_signature(
        &pool,
        w.owner,
        UpdateSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
            group: Some(SignatureGroup::Wormhole),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 2);
    assert!(view.systems.iter().any(|s| s.solar_system_id().is_none()));
}

#[sqlx::test]
async fn removing_a_system_takes_the_holes_hanging_off_it(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    let far = place(&pool, w.owner, w.map_id, SYS_C).await;
    ghosting(&pool, w.owner, w.map_id).await;
    for sig in ["ABC-123", "DEF-456"] {
        scan(&pool, w.owner, w.map_id, SYS_A, sig).await;
    }

    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: home,
        },
    )
    .await
    .unwrap();

    // The scanned system is gone, and so are the holes that only existed as its far
    // sides. The unrelated system stays.
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1);
    assert_eq!(view.systems[0].id(), far);
    assert!(view.connections.is_empty());

    // And one undo puts the system and its holes back together.
    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 4);
    assert_eq!(
        view.systems
            .iter()
            .filter(|s| s.solar_system_id().is_none())
            .count(),
        2
    );
    assert_eq!(view.connections.len(), 2);
}

/// The same removal, made with a marquee across the chain: the holes hanging off what was
/// swept up go with it, and are not left floating.
#[sqlx::test]
async fn a_marquee_delete_takes_the_holes_with_it(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    let far = place(&pool, w.owner, w.map_id, SYS_C).await;
    ghosting(&pool, w.owner, w.map_id).await;
    scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    scan(&pool, w.owner, w.map_id, SYS_C, "DEF-456").await;

    remove_systems(
        &pool,
        w.owner,
        RemoveSystems {
            map_id: w.map_id,
            map_solar_system_ids: vec![home, far],
        },
    )
    .await
    .unwrap();

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert!(view.systems.is_empty(), "nothing was left behind");
    assert!(view.connections.is_empty());
}

#[sqlx::test]
async fn a_ghost_goes_with_the_connection_that_made_it(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    let connection = view.connections[0].id;

    remove_connection(
        &pool,
        w.owner,
        RemoveConnection {
            map_id: w.map_id,
            connection_id: connection,
        },
    )
    .await
    .unwrap();

    // That node is gone. What is not gone is the scan, which still says a hole is there,
    // so the map draws it again: a ghost is a rendering of a signature, and cutting the
    // edge is not how you say the hole was never found. Deleting the signature is.
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert!(view.systems.iter().all(|s| s.id() != ghost));
    assert_eq!(view.systems.len(), 2);
    assert_eq!(view.connections.len(), 1);

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
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1, "the hole had nothing else to be");
    assert!(view.connections.is_empty());
}

#[sqlx::test]
async fn a_real_system_left_without_connections_stays(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    let far = place(&pool, w.owner, w.map_id, SYS_B).await;
    wormholesystems::maps::connection::add_connection(
        &pool,
        w.owner,
        wormholesystems::maps::connection::AddConnection {
            map_id: w.map_id,
            from_system: home,
            to_system: far,
            kind: wormholesystems::maps::ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();

    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: home,
        },
    )
    .await
    .unwrap();

    // Somebody put that system on the map on purpose; losing its last hole is not a
    // reason to take it away.
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1);
    assert_eq!(view.systems[0].id(), far);
}

#[sqlx::test]
async fn a_hole_nobody_has_been_through_cannot_be_pinned(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

    // Pinning holds a node still, roots the tree layout and is passed over by every
    // sweep, which would leave this one behind when its connection goes.
    assert!(matches!(
        set_pinned(
            &pool,
            w.owner,
            SetPinned {
                map_id: w.map_id,
                map_solar_system_id: ghost,
                value: true,
            }
        )
        .await,
        Err(MapError::Validation(_)),
    ));

    // Once it is a system, it pins like any other.
    resolve_ghost_system(
        &pool,
        w.owner,
        ResolveGhostSystem {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            solar_system_id: Some(SYS_B),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    set_pinned(
        &pool,
        w.owner,
        SetPinned {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            value: true,
        },
    )
    .await
    .unwrap();
}

#[sqlx::test]
async fn removing_a_system_takes_the_signature_that_led_to_it(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

    // Taking the far side off the map takes the hole with it: the signature it was
    // scanned as is the same fact, and leaving it behind puts the node back on the next
    // paste.
    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: ghost,
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

    // And one undo brings the node, its edge and the signature back, still linked.
    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(sigs.len(), 1);
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.connections.len(), 1);
    assert_eq!(sigs[0].connection_id, Some(view.connections[0].id));
}

/// The jump dialog collects an alias and the hole's size and lifetime. When the signature
/// it names is already drawn as a ghost, resolving is the write that happens, so it has to
/// carry them: they used to be dropped and nothing the dialog was told took effect.
#[sqlx::test]
async fn resolving_applies_what_the_jump_learned(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

    let resolved = resolve_ghost_system(
        &pool,
        w.owner,
        ResolveGhostSystem {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            solar_system_id: Some(SYS_B),
            alias: Some("2b".into()),
            size: Some(WormholeSize::Medium),
            mass_status: Some(MassStatus::Critical),
            time_status: Some(TimeStatus::Critical),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(resolved.alias.as_deref(), Some("2b"));
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    let edge = &view.connections[0];
    assert_eq!(edge.size, Some(WormholeSize::Medium));
    assert_eq!(edge.mass_status, Some(MassStatus::Critical));
    assert_eq!(edge.time_status, Some(TimeStatus::Critical));

    // One jump, one undo: the node goes back to a ghost and takes the rest with it.
    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    let node = view.systems.iter().find(|s| s.id() == ghost).unwrap();
    assert_eq!(node.solar_system_id(), None, "back to a ghost");
    assert_eq!(node.alias(), None);
    // Back to unset, which is what an unflown hole is: not "stable", just unknown.
    assert_eq!(view.connections[0].mass_status, None);
    assert_eq!(view.connections[0].time_status, None);
}

/// A ghost is the far side of one hole. Delete the signature that described it and the
/// node has nothing left to mean, so it goes with the connection rather than sitting
/// there unreachable.
#[sqlx::test]
async fn deleting_the_signature_takes_its_ghost(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

    let outcome = remove_signature(
        &pool,
        w.owner,
        RemoveSignature {
            map_id: w.map_id,
            signature_pk: sig,
        },
    )
    .await
    .unwrap();

    assert_eq!(outcome.removed_placement_ids, vec![ghost]);
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert!(view.connections.is_empty());
    assert_eq!(
        view.systems.iter().map(|s| s.id()).collect::<Vec<_>>(),
        vec![home],
        "only the system someone actually placed is left"
    );
}

/// The same delete against a real system leaves it alone: someone put it there.
#[sqlx::test]
async fn deleting_the_signature_leaves_a_real_system(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    let far = place(&pool, w.owner, w.map_id, SYS_B).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let edge = wormholesystems::maps::connection::add_connection(
        &pool,
        w.owner,
        wormholesystems::maps::connection::AddConnection {
            map_id: w.map_id,
            from_system: home,
            to_system: far,
            kind: wormholesystems::maps::ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();
    wormholesystems::maps::signatures::link_signature(
        &pool,
        w.owner,
        wormholesystems::maps::signatures::LinkSignature {
            map_id: w.map_id,
            signature_pk: sig,
            connection_id: edge.id,
        },
    )
    .await
    .unwrap();

    let outcome = remove_signature(
        &pool,
        w.owner,
        RemoveSignature {
            map_id: w.map_id,
            signature_pk: sig,
        },
    )
    .await
    .unwrap();

    assert!(outcome.removed_placement_ids.is_empty());
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 2);
}

/// Taking the far side away from a signature does not make the hole go away: the scan
/// still says it is there, so the map draws it again as a node nobody has been through.
/// The new node is linked to the signature that raised it, the same as a freshly scanned
/// one, so it shows the signature id rather than sitting there blank.
#[sqlx::test]
async fn unassigning_the_far_side_draws_the_hole_again(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;
    resolve_ghost_system(
        &pool,
        w.owner,
        ResolveGhostSystem {
            map_id: w.map_id,
            map_solar_system_id: ghost,
            solar_system_id: Some(SYS_B),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    unlink_signature(
        &pool,
        w.owner,
        UnlinkSignature {
            map_id: w.map_id,
            signature_pk: sig,
        },
    )
    .await
    .unwrap();

    // The system someone flew to stays where it is, edge and all. The scan gets a node of
    // its own back.
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 3);
    let redrawn = ghost_for(&pool, w.map_id, sig).await;
    assert_ne!(redrawn, ghost, "the system it used to lead to is not it");
    let node = view.systems.iter().find(|s| s.id() == redrawn).unwrap();
    assert_eq!(node.solar_system_id(), None);

    // Linked, so the node knows which signature it is.
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert!(sigs[0].connection_id.is_some());
    let edge = view
        .connections
        .iter()
        .find(|c| c.to_system == redrawn)
        .unwrap();
    assert_eq!(sigs[0].connection_id, Some(edge.id));
    assert_eq!(edge.from_system, home);
}

/// Drawing unmapped holes is a map setting, and flipping it takes effect there and then.
#[sqlx::test]
async fn the_setting_draws_and_undraws_the_holes_already_scanned(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1, "off: a scan is only a scan");

    ghosting(&pool, w.owner, w.map_id).await;
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(
        view.systems.len(),
        2,
        "on: the hole scanned earlier is drawn"
    );
    assert_eq!(view.connections.len(), 1);
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert!(sigs[0].connection_id.is_some());

    update_map(
        &pool,
        w.owner,
        UpdateMap {
            map_id: w.map_id,
            ghost_unlinked_wormholes: Some(false),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Off again: the nodes go, and the scans they were drawn from stay.
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1);
    assert!(view.connections.is_empty());
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(sigs.len(), 1);
    assert_eq!(sigs[0].id, sig);
    assert_eq!(sigs[0].connection_id, None);
}

/// Everything an open client has seen so far on this map's channel.
fn seen(events: &mut tokio::sync::broadcast::Receiver<MapEvent>) -> Vec<MapEvent> {
    std::iter::from_fn(|| events.try_recv().ok()).collect()
}

/// The nodes a settings change draws or takes away are a change to the chain like any
/// other: open clients hear about them and the history says who did it. They stay out of
/// the undo path, so stepping back still walks the changes made before.
#[sqlx::test]
async fn the_setting_announces_and_records_the_nodes_it_changes(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let mut events = wormholesystems::realtime::hub().subscribe(w.map_id);

    ghosting(&pool, w.owner, w.map_id).await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;
    assert!(seen(&mut events).contains(&MapEvent::SystemAdded {
        map_id: w.map_id,
        map_solar_system_id: ghost,
    }));
    let history = list_history(&pool, w.owner, w.map_id).await.unwrap();
    let newest = &history.entries[0];
    assert_eq!(newest.kind, "map.ghosts");
    assert_eq!(newest.label, "drew the unmapped holes");
    assert!(!newest.is_step);

    update_map(
        &pool,
        w.owner,
        UpdateMap {
            map_id: w.map_id,
            ghost_unlinked_wormholes: Some(false),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(seen(&mut events).contains(&MapEvent::SystemRemoved {
        map_id: w.map_id,
        map_solar_system_id: ghost,
    }));
    let history = list_history(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(
        history.entries[0].label,
        "took the unmapped holes off the map"
    );

    // A settings change that moves no node leaves the history alone.
    update_map(
        &pool,
        w.owner,
        UpdateMap {
            map_id: w.map_id,
            name: Some("Renamed".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let after = list_history(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(after.entries.len(), history.entries.len());

    // Undo steps back over the scan and then the placement, and redo brings both back.
    let body = MapIdBody { map_id: w.map_id };
    undo(&pool, w.owner, body.clone()).await.unwrap();
    assert!(
        list_signatures(&pool, w.owner, w.map_id)
            .await
            .unwrap()
            .is_empty()
    );
    undo(&pool, w.owner, body.clone()).await.unwrap();
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert!(view.systems.is_empty());
    redo(&pool, w.owner, body.clone()).await.unwrap();
    redo(&pool, w.owner, body).await.unwrap();
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1);
    assert_eq!(view.systems[0].id(), home);
    assert_eq!(
        list_signatures(&pool, w.owner, w.map_id).await.unwrap()[0].id,
        sig
    );
}

/// Saying a signature was never a wormhole after all takes its node with it: the hole it
/// was drawn for has stopped existing.
#[sqlx::test]
async fn retyping_a_wormhole_as_something_else_takes_its_node(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    ghost_for(&pool, w.map_id, sig).await;

    update_signature(
        &pool,
        w.owner,
        UpdateSignature {
            map_id: w.map_id,
            signature_pk: sig,
            group: Some(SignatureGroup::Relic),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1);
    assert!(view.connections.is_empty());
}

/// Nothing is joined to an unmapped hole by hand. An edge out of it would claim the
/// system on its far side leads somewhere, which is the one thing nobody knows yet, and it
/// would give the node a second reason to exist that its signature never granted.
#[sqlx::test]
async fn nothing_can_be_connected_to_a_hole_nobody_has_been_through(pool: PgPool) {
    let w = world(&pool).await;
    let other = place(&pool, w.owner, w.map_id, SYS_C).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

    for (from, to) in [(other, ghost), (ghost, other)] {
        assert!(matches!(
            wormholesystems::maps::connection::add_connection(
                &pool,
                w.owner,
                wormholesystems::maps::connection::AddConnection {
                    map_id: w.map_id,
                    from_system: from,
                    to_system: to,
                    kind: wormholesystems::maps::ConnectionType::Wormhole,
                    size: None,
                },
            )
            .await,
            Err(MapError::Validation(_)),
        ));
    }

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.connections.len(), 1, "only the hole's own edge");
}

/// The node and the scan are one fact, so undoing the delete brings both back, linked as
/// they were. The row naming its signature is what makes that possible: before, the node
/// was found through its connection and the connection had already gone.
#[sqlx::test]
async fn undoing_a_signature_delete_brings_its_node_back(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

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

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    let back = view.systems.iter().find(|s| s.id() == ghost).unwrap();
    assert_eq!(back.solar_system_id(), None);
    assert_eq!(view.connections.len(), 1);
    assert_eq!(view.connections[0].from_system, home);
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(sigs.len(), 1);
    assert_eq!(
        sigs[0].connection_id,
        Some(view.connections[0].id),
        "and still linked to the edge it raised"
    );
}

// The command refuses an edge to a ghost however it is reached, not only from the API: what
// is beyond an unmapped hole is the one thing nobody knows yet.
#[sqlx::test]
async fn nothing_connects_to_a_ghost_but_the_scan_that_raised_it(pool: PgPool) {
    use wormholesystems::maps::ConnectionType;
    use wormholesystems::maps::connection::AddConnection;

    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    let other = place(&pool, w.owner, w.map_id, SYS_B).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let sig = scan(&pool, w.owner, w.map_id, SYS_A, "ABC-123").await;
    let ghost = ghost_for(&pool, w.map_id, sig).await;

    let err = wormholesystems::maps::run(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: other,
            to_system: ghost,
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, MapError::Validation(_)), "{err:?}");
}

/// Walking a removal forward again on a map that draws unmapped holes.
///
/// The replay does not pass through the states the original did. Taking the scan removes
/// the hole, which leaves its far side with no edges for a moment -- long enough for
/// `delete_if_orphan` to clear it -- and the restore that follows was holding an edge back
/// to it. The redo died on the foreign key.
#[sqlx::test]
async fn a_removal_can_be_walked_forward_again(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    ghosting(&pool, w.owner, w.map_id).await;
    let conn = connect(&pool, w.owner, w.map_id, a, b).await;

    // A scan on each side of the hole, both linked to it.
    let mut scans = Vec::new();
    for (system, id) in [(SYS_A, "AAA-111"), (SYS_B, "BBB-222")] {
        let sig = add_signature(
            &pool,
            w.owner,
            AddSignature {
                map_id: w.map_id,
                solar_system_id: system,
                signature_id: id.into(),
                group: SignatureGroup::Wormhole,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        link_signature(
            &pool,
            w.owner,
            LinkSignature {
                map_id: w.map_id,
                signature_pk: sig.id,
                connection_id: conn.id,
            },
        )
        .await
        .unwrap();
        scans.push(sig.id);
    }

    remove_signature(
        &pool,
        w.owner,
        RemoveSignature {
            map_id: w.map_id,
            signature_pk: scans[0],
        },
    )
    .await
    .unwrap();
    let after_removal = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();

    let body = MapIdBody { map_id: w.map_id };
    undo(&pool, w.owner, body.clone()).await.unwrap();
    redo(&pool, w.owner, body).await.unwrap();

    // And it lands where the removal first left it, rather than merely surviving.
    let after_redo = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    let shape = |view: &wormholesystems::maps::MapView| {
        let mut systems: Vec<Option<i64>> =
            view.systems.iter().map(|s| s.solar_system_id()).collect();
        systems.sort();
        let mut edges: Vec<(i64, i64)> = view
            .connections
            .iter()
            .map(|c| (c.from_system, c.to_system))
            .collect();
        edges.sort();
        (systems, edges)
    };
    assert_eq!(shape(&after_redo), shape(&after_removal));
}
