//! Signatures: CRUD, linking a wormhole sig to a connection, and the connection<->signature
//! state sync (worst-wins on link, verbatim propagation on edit).

mod common;

use common::catalog::{C247, K162, seed_catalog};
use common::map::{connect, place};
use common::{SYS_A, SYS_B, member_with_role, world};
use sqlx::PgPool;
use wormholesystems::maps::connection::{
    AddConnection, SetConnectionStatus, add_connection, set_connection_status,
};
use wormholesystems::maps::map::{GetMap, get_map};
use wormholesystems::maps::signatures::{
    AddSignature, LinkSignature, RemoveSignature, RemoveSignatures, Signature, UnlinkSignature,
    UpdateSignature, add_signature, link_signature, list_signatures, remove_signature,
    remove_signatures, unlink_signature, update_signature,
};
use wormholesystems::maps::{
    Actor, MapError, MassStatus, Role, SignatureGroup, TimeStatus, WormholeSize,
};

#[sqlx::test]
async fn add_validates_and_persists(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    let viewer = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Viewer).await;

    // A viewer can't scan.
    assert!(matches!(
        add_signature(
            &pool,
            viewer,
            AddSignature {
                map_id: w.map_id,
                solar_system_id: SYS_A,
                signature_id: "ABC-123".into(),
                group: SignatureGroup::Wormhole,
                ..Default::default()
            }
        )
        .await,
        Err(MapError::Forbidden),
    ));

    let sig = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Wormhole,
            mass_status: Some(MassStatus::Critical),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(sig.signature_id, "ABC-123");
    assert_eq!(sig.group, SignatureGroup::Wormhole);
    assert_eq!(sig.mass_status, Some(MassStatus::Critical));
    assert_eq!(sig.connection_id, None);

    // A non-wormhole sig may not carry wormhole state.
    assert!(matches!(
        add_signature(
            &pool,
            w.owner,
            AddSignature {
                map_id: w.map_id,
                solar_system_id: SYS_A,
                signature_id: "DAT-001".into(),
                group: SignatureGroup::Data,
                mass_status: Some(MassStatus::Stable),
                ..Default::default()
            }
        )
        .await,
        Err(MapError::Validation(_)),
    ));

    // Duplicate id in the same system → Conflict.
    assert!(matches!(
        add_signature(
            &pool,
            w.owner,
            AddSignature {
                map_id: w.map_id,
                solar_system_id: SYS_A,
                signature_id: "ABC-123".into(),
                group: SignatureGroup::Wormhole,
                ..Default::default()
            }
        )
        .await,
        Err(MapError::Conflict(_)),
    ));

    // A signature for a system not placed on the map → Validation.
    assert!(matches!(
        add_signature(
            &pool,
            w.owner,
            AddSignature {
                map_id: w.map_id,
                solar_system_id: SYS_B,
                signature_id: "ZZZ-999".into(),
                group: SignatureGroup::Wormhole,
                ..Default::default()
            }
        )
        .await,
        Err(MapError::Validation(_)),
    ));
}

#[sqlx::test]
async fn update_and_remove(pool: PgPool) {
    let w = world(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;
    let sig = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Wormhole,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let updated = update_signature(
        &pool,
        w.owner,
        UpdateSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
            name: Some(Some("C247".into())),
            time_status: Some(Some(TimeStatus::Eol)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.name.as_deref(), Some("C247"));
    assert_eq!(updated.time_status, Some(TimeStatus::Eol));

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

    // Removing an unknown signature → NotFound.
    assert!(matches!(
        remove_signature(
            &pool,
            w.owner,
            RemoveSignature {
                map_id: w.map_id,
                signature_pk: 4242
            }
        )
        .await,
        Err(MapError::NotFound),
    ));
}

/// The core sync contract: linking merges to the worst state per field (crossing in *both*
/// directions), then later edits propagate verbatim — including downgrades.
#[sqlx::test]
async fn link_merges_worst_then_edits_propagate(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let conn = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: a,
            to_system: b,
            kind: wormholesystems::maps::ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();

    // Connection marked: mass critical, time stable.
    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn.id,
            mass_status: Some(Some(MassStatus::Critical)),
            time_status: Some(Some(TimeStatus::Stable)),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // A scanned wormhole sig in A: mass stable, time eol — disagrees with the connection on
    // *both* fields, each side worse on a different one.
    let sig = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Wormhole,
            mass_status: Some(MassStatus::Stable),
            time_status: Some(TimeStatus::Eol),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Link → merge: mass = critical (from the connection), time = eol (from the sig).
    let linked = link_signature(
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
    assert_eq!(linked.connection_id, Some(conn.id));
    assert_eq!(
        linked.mass_status,
        Some(MassStatus::Critical),
        "sig pulled up from connection"
    );
    assert_eq!(linked.time_status, Some(TimeStatus::Eol));
    // And the connection adopted the sig's worse time.
    let c = connection_state(&pool, w.owner, w.map_id, conn.id).await;
    assert_eq!(c.mass_status, Some(MassStatus::Critical));
    assert_eq!(
        c.time_status,
        Some(TimeStatus::Eol),
        "connection pulled up from sig"
    );

    // Downgrade the connection to fully stable → the sig follows (verbatim, not worst-wins).
    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn.id,
            mass_status: Some(Some(MassStatus::Stable)),
            time_status: Some(Some(TimeStatus::Stable)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let s = only_signature(&pool, w.owner, w.map_id).await;
    assert_eq!(
        s.mass_status,
        Some(MassStatus::Stable),
        "downgrade propagated to sig"
    );
    assert_eq!(s.time_status, Some(TimeStatus::Stable));

    // Edit from the signature side → the connection follows.
    update_signature(
        &pool,
        w.owner,
        UpdateSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
            mass_status: Some(Some(MassStatus::Reduced)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let c = connection_state(&pool, w.owner, w.map_id, conn.id).await;
    assert_eq!(
        c.mass_status,
        Some(MassStatus::Reduced),
        "sig edit propagated to connection"
    );
}

#[sqlx::test]
async fn link_validations_and_unlink(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let conn = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: a,
            to_system: b,
            kind: wormholesystems::maps::ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();

    // A non-wormhole sig can't link.
    let data = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "DAT-001".into(),
            group: SignatureGroup::Data,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        link_signature(
            &pool,
            w.owner,
            LinkSignature {
                map_id: w.map_id,
                signature_pk: data.id,
                connection_id: conn.id
            }
        )
        .await,
        Err(MapError::Validation(_)),
    ));

    // A wormhole sig links and then unlinks, keeping its state as a standalone sig.
    let sig = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Wormhole,
            time_status: Some(TimeStatus::Eol),
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
    let unlinked = unlink_signature(
        &pool,
        w.owner,
        UnlinkSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
        },
    )
    .await
    .unwrap();
    assert_eq!(unlinked.connection_id, None);
    assert_eq!(
        unlinked.time_status,
        Some(TimeStatus::Eol),
        "state survives unlink"
    );
}

/// An identified wormhole type dictates the group's size: linking applies it, manual edits
/// on either side are overridden, and only losing the identification (retyped to K162,
/// unlinked) lets a manual size stick again, keeping the size the group had.
#[sqlx::test]
async fn identified_type_locks_size(pool: PgPool) {
    let w = world(&pool).await;
    seed_catalog(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let conn = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: a,
            to_system: b,
            kind: wormholesystems::maps::ConnectionType::Wormhole,
            size: Some(WormholeSize::Large),
        },
    )
    .await
    .unwrap();
    let sig = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Wormhole,
            signature_type_id: Some(C247),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let linked = link_signature(
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
    assert_eq!(
        linked.size,
        Some(WormholeSize::Medium),
        "C247 dictates medium"
    );
    let c = connection_state(&pool, w.owner, w.map_id, conn.id).await;
    assert_eq!(c.size, Some(WormholeSize::Medium));

    // Manual edits on either side are overridden.
    let c = set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn.id,
            size: Some(Some(WormholeSize::Small)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        c.size,
        Some(WormholeSize::Medium),
        "connection edit overridden"
    );
    let s = update_signature(
        &pool,
        w.owner,
        UpdateSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
            size: Some(Some(WormholeSize::Xl)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        s.size,
        Some(WormholeSize::Medium),
        "signature edit overridden"
    );
    let c = connection_state(&pool, w.owner, w.map_id, conn.id).await;
    assert_eq!(c.size, Some(WormholeSize::Medium));

    // Retyping to K162 lifts the lock: the size stays, and an edit sticks again.
    let s = update_signature(
        &pool,
        w.owner,
        UpdateSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
            signature_type_id: Some(Some(K162)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        s.size,
        Some(WormholeSize::Medium),
        "lifting the lock keeps the size"
    );
    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn.id,
            size: Some(Some(WormholeSize::Small)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let s = only_signature(&pool, w.owner, w.map_id).await;
    assert_eq!(
        s.size,
        Some(WormholeSize::Small),
        "unlocked edit propagates"
    );

    // Identifying the hole again reapplies the lock over the manual size.
    update_signature(
        &pool,
        w.owner,
        UpdateSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
            signature_type_id: Some(Some(C247)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let c = connection_state(&pool, w.owner, w.map_id, conn.id).await;
    assert_eq!(c.size, Some(WormholeSize::Medium), "retyping relocks");

    // Unlinking lifts it as well: the connection keeps medium and accepts an edit.
    unlink_signature(
        &pool,
        w.owner,
        UnlinkSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
        },
    )
    .await
    .unwrap();
    let c = connection_state(&pool, w.owner, w.map_id, conn.id).await;
    assert_eq!(
        c.size,
        Some(WormholeSize::Medium),
        "unlinking keeps the size"
    );
    let c = set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: conn.id,
            size: Some(Some(WormholeSize::Xl)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(c.size, Some(WormholeSize::Xl), "unlinked edit sticks");
}

/// next#7: two holes in one system leading to the same place are two connections, and each
/// keeps its own state. Pointing a second signature at a connection another already claims
/// hands the connection over rather than putting both in one sync group.
#[sqlx::test]
async fn a_connection_takes_one_signature_per_system(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let hole = |from: i64, to: i64| AddConnection {
        map_id: w.map_id,
        from_system: from,
        to_system: to,
        kind: wormholesystems::maps::ConnectionType::Wormhole,
        size: None,
    };
    let first = add_connection(&pool, w.owner, hole(a, b)).await.unwrap();
    let second = add_connection(&pool, w.owner, hole(a, b)).await.unwrap();

    let scan = |id: &str| AddSignature {
        map_id: w.map_id,
        solar_system_id: SYS_A,
        signature_id: id.into(),
        group: SignatureGroup::Wormhole,
        ..Default::default()
    };
    let one = add_signature(&pool, w.owner, scan("AAA-111"))
        .await
        .unwrap();
    let two = add_signature(&pool, w.owner, scan("BBB-222"))
        .await
        .unwrap();
    let link = |signature_pk: i64, connection_id: i64| LinkSignature {
        map_id: w.map_id,
        signature_pk,
        connection_id,
    };
    link_signature(&pool, w.owner, link(one.id, first.id))
        .await
        .unwrap();
    link_signature(&pool, w.owner, link(two.id, second.id))
        .await
        .unwrap();

    // Marking one hole says nothing about the other, though both end in the same system.
    set_connection_status(
        &pool,
        w.owner,
        SetConnectionStatus {
            map_id: w.map_id,
            connection_id: first.id,
            mass_status: Some(Some(MassStatus::Critical)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        connection_state(&pool, w.owner, w.map_id, second.id)
            .await
            .mass_status,
        None,
        "the other hole is untouched"
    );
    let scanned = |sigs: &[Signature], id: &str| {
        sigs.iter()
            .find(|s| s.signature_id == id)
            .expect("signature present")
            .clone()
    };
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(
        scanned(&sigs, "AAA-111").mass_status,
        Some(MassStatus::Critical)
    );
    assert_eq!(
        scanned(&sigs, "BBB-222").mass_status,
        None,
        "the other signature is not in that group"
    );

    // Saying the second signature is the first hole takes the connection off the first.
    link_signature(&pool, w.owner, link(two.id, first.id))
        .await
        .unwrap();
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(scanned(&sigs, "BBB-222").connection_id, Some(first.id));
    assert_eq!(
        scanned(&sigs, "AAA-111").connection_id,
        None,
        "displaced, and free to take the hole it really is"
    );

    // Undoing that hands the connection back rather than leaving it on the new claimant,
    // and the claimant goes back to the hole it was on before.
    wormholesystems::maps::history::undo(
        &pool,
        w.owner,
        wormholesystems::maps::history::MapIdBody { map_id: w.map_id },
    )
    .await
    .unwrap();
    let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
    assert_eq!(scanned(&sigs, "AAA-111").connection_id, Some(first.id));
    assert_eq!(scanned(&sigs, "BBB-222").connection_id, Some(second.id));
}

/// Bulk deleting the signature of a system's only connection takes the connection and the
/// stranded far end, but never the system the signatures were in, even when that leaves it
/// with no connections.
#[sqlx::test]
async fn bulk_remove_keeps_the_origin_system(pool: PgPool) {
    let w = world(&pool).await;
    let origin = place(&pool, w.owner, w.map_id, SYS_A).await;
    let target = place(&pool, w.owner, w.map_id, SYS_B).await;
    let conn = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: origin,
            to_system: target,
            kind: wormholesystems::maps::ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();
    let scan = |id: &str| AddSignature {
        map_id: w.map_id,
        solar_system_id: SYS_A,
        signature_id: id.into(),
        group: SignatureGroup::Wormhole,
        ..Default::default()
    };
    let connected = add_signature(&pool, w.owner, scan("AAA-111"))
        .await
        .unwrap();
    let remaining = add_signature(&pool, w.owner, scan("BBB-222"))
        .await
        .unwrap();
    link_signature(
        &pool,
        w.owner,
        LinkSignature {
            map_id: w.map_id,
            signature_pk: connected.id,
            connection_id: conn.id,
        },
    )
    .await
    .unwrap();

    let out = remove_signatures(
        &pool,
        w.owner,
        RemoveSignatures {
            map_id: w.map_id,
            signature_pks: vec![connected.id],
        },
    )
    .await
    .unwrap();
    assert_eq!(out.removed_connection_ids, vec![conn.id]);
    assert_eq!(out.removed_placement_ids, vec![target]);

    let placed: Vec<i64> =
        sqlx::query_scalar("select id from map_solar_systems where map_id = $1 order by id")
            .bind(w.map_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(placed, vec![origin]);
    let left: Vec<i64> = list_signatures(&pool, w.owner, w.map_id)
        .await
        .unwrap()
        .iter()
        .map(|s| s.id)
        .collect();
    assert_eq!(left, vec![remaining.id]);
}

/// Read the connection's current state back through `get_map`.
async fn connection_state(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    connection_id: i64,
) -> wormholesystems::maps::MapConnection {
    get_map(pool, actor, GetMap { map_id })
        .await
        .unwrap()
        .connections
        .into_iter()
        .find(|c| c.id == connection_id)
        .expect("connection present")
}

async fn only_signature(pool: &PgPool, actor: Actor, map_id: i64) -> Signature {
    let mut sigs = list_signatures(pool, actor, map_id).await.unwrap();
    assert_eq!(sigs.len(), 1);
    sigs.pop().unwrap()
}

/// Wind a signature's edit stamp back a day.
async fn backdate(pool: &PgPool, signature_pk: i64) {
    sqlx::query("update signatures set updated_at = now() - interval '1 day' where id = $1")
        .bind(signature_pk)
        .execute(pool)
        .await
        .unwrap();
}

/// Whether the edit stamp has moved since it was wound back.
async fn touched(pool: &PgPool, signature_pk: i64) -> bool {
    sqlx::query_scalar(
        "select updated_at > now() - interval '1 minute' from signatures where id = $1",
    )
    .bind(signature_pk)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Linking and unlinking are edits to the signature like any other, the same as the unlink
/// a link does to make room, whether or not the sync had any state to change.
#[sqlx::test]
async fn linking_and_unlinking_touch_the_signature(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let conn = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: a,
            to_system: b,
            kind: wormholesystems::maps::ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();
    let sig = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Wormhole,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    backdate(&pool, sig.id).await;
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
    assert!(touched(&pool, sig.id).await, "linking");

    backdate(&pool, sig.id).await;
    unlink_signature(
        &pool,
        w.owner,
        UnlinkSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
        },
    )
    .await
    .unwrap();
    assert!(touched(&pool, sig.id).await, "unlinking");
}

/// The last scan on one side of a hole takes the hole with it. Undoing that has to bring
/// the hole back before the scan, and the scan on the other side has to get its link back.
#[sqlx::test]
async fn undoing_the_removal_of_a_holes_last_scan_brings_the_hole_back(pool: PgPool) {
    use wormholesystems::maps::history::{MapIdBody, redo, undo};

    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B).await;
    let conn = connect(&pool, w.owner, w.map_id, a, b).await;
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

    for remove in [true, false] {
        if remove {
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
        } else {
            remove_signatures(
                &pool,
                w.owner,
                RemoveSignatures {
                    map_id: w.map_id,
                    signature_pks: vec![scans[0]],
                },
            )
            .await
            .unwrap();
        }
        let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
            .await
            .unwrap();
        assert!(view.connections.is_empty(), "the hole went with its scan");

        undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
            .await
            .unwrap();
        let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
            .await
            .unwrap();
        assert_eq!(view.connections[0].id, conn.id);
        let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
        assert!(sigs.iter().all(|s| s.connection_id == Some(conn.id)));

        redo(&pool, w.owner, MapIdBody { map_id: w.map_id })
            .await
            .unwrap();
        undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
            .await
            .unwrap();
        let sigs = list_signatures(&pool, w.owner, w.map_id).await.unwrap();
        assert_eq!(sigs.len(), 2);
        assert!(sigs.iter().all(|s| s.connection_id == Some(conn.id)));
    }
}
