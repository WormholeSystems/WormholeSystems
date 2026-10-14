//! Paste reconciliation (legacy upsert semantics), the delete cascades, and expiry.

mod common;

use common::catalog::seed_catalog;
use common::map::place;
use common::{SYS_A, SYS_B, world};
use sqlx::PgPool;
use wormholesystems::maps::connection::{AddConnection, add_connection};
use wormholesystems::maps::map::{GetMap, get_map};
use wormholesystems::maps::signatures::{
    AddSignature, LinkSignature, PasteSignatures, PastedSignature, RemoveSignature,
    RemoveSignatures, Signature, UpdateSignature, add_signature, expire_signatures, link_signature,
    list_signatures, paste_signatures, remove_signature, remove_signatures, update_signature,
};
use wormholesystems::maps::solar_system::{RemoveSystem, remove_system};
use wormholesystems::maps::{
    Actor, ConnectionType, MapError, MapSystemView, SignatureGroup, WormholeSize,
};

async fn sig_by_id(pool: &PgPool, actor: Actor, map_id: i64, sid: &str) -> Option<Signature> {
    list_signatures(pool, actor, map_id)
        .await
        .unwrap()
        .into_iter()
        .find(|s| s.signature_id == sid)
}

#[sqlx::test]
async fn paste_upserts_without_deleting(pool: PgPool) {
    let w = world(&pool).await;
    seed_catalog(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;

    // An existing typed wormhole sig, plus a stray one the next paste won't mention.
    add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "WHX-001".into(),
            group: SignatureGroup::Wormhole,
            signature_type_id: Some(500),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "OLD-001".into(),
            group: SignatureGroup::Combat,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    paste_signatures(
        &pool,
        w.owner,
        PasteSignatures {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signatures: vec![
                // Repaste of the wormhole: no type from the scanner, name present.
                PastedSignature {
                    signature_id: "WHX-001".into(),
                    group: Some(SignatureGroup::Wormhole),
                    signature_type_id: None,
                    name: Some("Unstable Wormhole".into()),
                },
                // New site with a matched catalog type.
                PastedSignature {
                    signature_id: "DAT-001".into(),
                    group: Some(SignatureGroup::Data),
                    signature_type_id: Some(600),
                    name: None,
                },
                // Unclassified row: no group at all.
                PastedSignature {
                    signature_id: "UNK-001".into(),
                    group: None,
                    signature_type_id: None,
                    name: None,
                },
            ],
        },
    )
    .await
    .unwrap();

    // The wormhole kept its manually chosen type; the stray sig was NOT deleted.
    let wh = sig_by_id(&pool, w.owner, w.map_id, "WHX-001")
        .await
        .unwrap();
    assert_eq!(wh.signature_type_id, Some(500));
    assert!(
        sig_by_id(&pool, w.owner, w.map_id, "OLD-001")
            .await
            .is_some()
    );
    let dat = sig_by_id(&pool, w.owner, w.map_id, "DAT-001")
        .await
        .unwrap();
    assert_eq!(dat.signature_type_id, Some(600));
    let unk = sig_by_id(&pool, w.owner, w.map_id, "UNK-001")
        .await
        .unwrap();
    assert_eq!(unk.group, SignatureGroup::Unknown);

    // A later classified repaste of the unknown row keeps working; an unclassified
    // repaste of a classified row keeps the existing group.
    paste_signatures(
        &pool,
        w.owner,
        PasteSignatures {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signatures: vec![PastedSignature {
                signature_id: "DAT-001".into(),
                group: None,
                signature_type_id: None,
                name: None,
            }],
        },
    )
    .await
    .unwrap();
    let dat = sig_by_id(&pool, w.owner, w.map_id, "DAT-001")
        .await
        .unwrap();
    assert_eq!(dat.group, SignatureGroup::Data);
    assert_eq!(dat.signature_type_id, Some(600));
}

#[sqlx::test]
async fn paste_recategorize_clears_link_and_validates_ids(pool: PgPool) {
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
            kind: ConnectionType::Wormhole,
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
            signature_id: "WHX-002".into(),
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

    // The scanner now says it's a data site: link dropped, connection itself survives.
    paste_signatures(
        &pool,
        w.owner,
        PasteSignatures {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signatures: vec![PastedSignature {
                signature_id: "WHX-002".into(),
                group: Some(SignatureGroup::Data),
                signature_type_id: None,
                name: Some("Some Data Site".into()),
            }],
        },
    )
    .await
    .unwrap();
    let sig = sig_by_id(&pool, w.owner, w.map_id, "WHX-002")
        .await
        .unwrap();
    assert_eq!(sig.group, SignatureGroup::Data);
    assert_eq!(sig.connection_id, None);
    assert_eq!(sig.name.as_deref(), Some("Some Data Site"));
    let conn_exists: bool =
        sqlx::query_scalar("select exists(select 1 from map_connections where id = $1)")
            .bind(conn.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(conn_exists);

    // Malformed scanner ids are rejected.
    let err = paste_signatures(
        &pool,
        w.owner,
        PasteSignatures {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signatures: vec![PastedSignature {
                signature_id: "TOOLONG-123".into(),
                group: None,
                signature_type_id: None,
                name: None,
            }],
        },
    )
    .await;
    assert!(matches!(err, Err(MapError::Validation(_))));
}

/// A paste that identifies a linked hole's type sets the connection's size from it.
#[sqlx::test]
async fn paste_identifying_a_linked_hole_sets_its_size(pool: PgPool) {
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
            kind: ConnectionType::Wormhole,
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
            signature_id: "WHX-001".into(),
            group: SignatureGroup::Wormhole,
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
        Some(WormholeSize::Large),
        "untyped, nothing locks"
    );

    paste_signatures(
        &pool,
        w.owner,
        PasteSignatures {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signatures: vec![PastedSignature {
                signature_id: "WHX-001".into(),
                group: Some(SignatureGroup::Wormhole),
                signature_type_id: Some(510),
                name: None,
            }],
        },
    )
    .await
    .unwrap();

    let sig = sig_by_id(&pool, w.owner, w.map_id, "WHX-001")
        .await
        .unwrap();
    assert_eq!(sig.signature_type_id, Some(510));
    assert_eq!(sig.size, Some(WormholeSize::Medium), "C247 dictates medium");
    let connection = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap()
        .connections
        .into_iter()
        .find(|c| c.id == conn.id)
        .unwrap();
    assert_eq!(connection.size, Some(WormholeSize::Medium));
}

#[sqlx::test]
async fn update_group_change_clears_type_link_and_state(pool: PgPool) {
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
            kind: ConnectionType::Wormhole,
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
            signature_id: "WHX-003".into(),
            group: SignatureGroup::Wormhole,
            signature_type_id: Some(500),
            time_status: Some(wormholesystems::maps::TimeStatus::Eol),
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

    let updated = update_signature(
        &pool,
        w.owner,
        UpdateSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
            group: Some(SignatureGroup::Relic),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.group, SignatureGroup::Relic);
    assert_eq!(updated.signature_type_id, None);
    assert_eq!(updated.connection_id, None);
    assert_eq!(updated.time_status, None);

    // A type from the wrong category is rejected.
    let err = update_signature(
        &pool,
        w.owner,
        UpdateSignature {
            map_id: w.map_id,
            signature_pk: sig.id,
            signature_type_id: Some(Some(500)),
            ..Default::default()
        },
    )
    .await;
    assert!(matches!(err, Err(MapError::Validation(_))));
}

#[sqlx::test]
async fn remove_cascades_connection_unless_side_survivor(pool: PgPool) {
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
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();

    let mut pks = Vec::new();
    for sid in ["WHX-004", "WHX-005"] {
        let sig = add_signature(
            &pool,
            w.owner,
            AddSignature {
                map_id: w.map_id,
                solar_system_id: SYS_A,
                signature_id: sid.into(),
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
        pks.push(sig.id);
    }

    // First delete: a same-side sibling still references the connection → it survives.
    let out = remove_signature(
        &pool,
        w.owner,
        RemoveSignature {
            map_id: w.map_id,
            signature_pk: pks[0],
        },
    )
    .await
    .unwrap();
    assert_eq!(out.removed_connection_id, None);

    // Second delete: last same-side sig → connection goes.
    let out = remove_signature(
        &pool,
        w.owner,
        RemoveSignature {
            map_id: w.map_id,
            signature_pk: pks[1],
        },
    )
    .await
    .unwrap();
    assert_eq!(out.removed_connection_id, Some(conn.id));
    let conn_exists: bool =
        sqlx::query_scalar("select exists(select 1 from map_connections where id = $1)")
            .bind(conn.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!conn_exists);
}

#[sqlx::test]
async fn bulk_remove_cascades_and_cleans_orphan_endpoints(pool: PgPool) {
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
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();
    // Pin the near end so only the far end is orphan-eligible.
    sqlx::query("update map_solar_systems set is_pinned = true where id = $1")
        .bind(a)
        .execute(&pool)
        .await
        .unwrap();

    let sig = add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "WHX-006".into(),
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

    let out = remove_signatures(
        &pool,
        w.owner,
        RemoveSignatures {
            map_id: w.map_id,
            signature_pks: vec![sig.id],
        },
    )
    .await
    .unwrap();
    assert_eq!(out.systems, vec![SYS_A]);
    assert_eq!(out.removed_connection_ids, vec![conn.id]);
    assert_eq!(out.removed_placement_ids, vec![b]);

    let a_exists: bool =
        sqlx::query_scalar("select exists(select 1 from map_solar_systems where id = $1)")
            .bind(a)
            .fetch_one(&pool)
            .await
            .unwrap();
    let b_exists: bool =
        sqlx::query_scalar("select exists(select 1 from map_solar_systems where id = $1)")
            .bind(b)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(a_exists, "pinned endpoint must survive");
    assert!(!b_exists, "orphaned endpoint must be removed");
}

#[sqlx::test]
async fn expiry_purges_stale_unlinked_signatures(pool: PgPool) {
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
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();

    for (sid, group) in [
        ("WHX-OLD", SignatureGroup::Wormhole),
        ("WHX-LNK", SignatureGroup::Wormhole),
        ("SIT-OLD", SignatureGroup::Data),
        ("SIT-NEW", SignatureGroup::Data),
    ] {
        add_signature(
            &pool,
            w.owner,
            AddSignature {
                map_id: w.map_id,
                solar_system_id: SYS_A,
                signature_id: sid.into(),
                group,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }
    let linked = sig_by_id(&pool, w.owner, w.map_id, "WHX-LNK")
        .await
        .unwrap();
    link_signature(
        &pool,
        w.owner,
        LinkSignature {
            map_id: w.map_id,
            signature_pk: linked.id,
            connection_id: conn.id,
        },
    )
    .await
    .unwrap();

    // Age everything past the wormhole cutoff, and the old site past the site cutoff.
    sqlx::query(
        "update signatures set created_at = now() - interval '4 days',
                               updated_at = now() - interval '4 days'",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "update signatures set updated_at = now() - interval '8 days'
         where signature_id = 'SIT-OLD'",
    )
    .execute(&pool)
    .await
    .unwrap();

    let pairs = expire_signatures(&pool).await.unwrap();
    assert_eq!(pairs, vec![(w.map_id, SYS_A)]);

    assert!(
        sig_by_id(&pool, w.owner, w.map_id, "WHX-OLD")
            .await
            .is_none()
    );
    assert!(
        sig_by_id(&pool, w.owner, w.map_id, "SIT-OLD")
            .await
            .is_none()
    );
    assert!(
        sig_by_id(&pool, w.owner, w.map_id, "WHX-LNK")
            .await
            .is_some(),
        "linked wormholes never expire"
    );
    assert!(
        sig_by_id(&pool, w.owner, w.map_id, "SIT-NEW")
            .await
            .is_some(),
        "sites under 7 days stay"
    );
}

#[sqlx::test]
async fn paste_files_faction_warfare_sites_under_their_own_group(pool: PgPool) {
    let w = world(&pool).await;
    seed_catalog(&pool).await;
    place(&pool, w.owner, w.map_id, SYS_A).await;

    // The scanner labels these anomalies "Factional Warfare Site - Combat Site" with a
    // type name that rotates with the war, so the parser sends the group and the raw name.
    paste_signatures(
        &pool,
        w.owner,
        PasteSignatures {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signatures: vec![PastedSignature {
                signature_id: "BUH-704".into(),
                group: Some(SignatureGroup::FactionWarfare),
                signature_type_id: None,
                name: Some("Minmatar Large NVY-1".into()),
            }],
        },
    )
    .await
    .unwrap();

    let sig = sig_by_id(&pool, w.owner, w.map_id, "BUH-704")
        .await
        .unwrap();
    assert_eq!(sig.group, SignatureGroup::FactionWarfare);
    assert_eq!(sig.signature_type_id, None);
    assert_eq!(sig.name.as_deref(), Some("Minmatar Large NVY-1"));
}

/// The placement's `signatures_pasted_at`, as the map read shows it.
async fn pasted_at(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    placement: i64,
) -> Option<chrono::DateTime<chrono::Utc>> {
    let system = get_map(pool, actor, GetMap { map_id })
        .await
        .unwrap()
        .systems
        .into_iter()
        .find(|s| s.id() == placement)
        .expect("placement on the map");
    match system {
        MapSystemView::System {
            signatures_pasted_at,
            ..
        } => signatures_pasted_at,
        MapSystemView::Ghost { .. } => panic!("a placed system, not a ghost"),
    }
}

async fn paste_one(pool: &PgPool, actor: Actor, map_id: i64, sid: &str) {
    paste_signatures(
        pool,
        actor,
        PasteSignatures {
            map_id,
            solar_system_id: SYS_A,
            signatures: vec![PastedSignature {
                signature_id: sid.into(),
                group: None,
                signature_type_id: None,
                name: None,
            }],
        },
    )
    .await
    .unwrap();
}

async fn undo(pool: &PgPool, actor: Actor, map_id: i64) {
    wormholesystems::maps::events_log::undo(
        pool,
        actor,
        wormholesystems::maps::events_log::MapIdBody { map_id },
    )
    .await
    .unwrap();
}

#[sqlx::test]
async fn a_paste_records_when_it_happened_and_undo_puts_the_old_time_back(pool: PgPool) {
    let w = world(&pool).await;
    let placement = place(&pool, w.owner, w.map_id, SYS_A).await;
    assert_eq!(pasted_at(&pool, w.owner, w.map_id, placement).await, None);

    paste_one(&pool, w.owner, w.map_id, "AAA-111").await;
    let first = pasted_at(&pool, w.owner, w.map_id, placement)
        .await
        .expect("set by the paste");

    // A hand edit is not a rescan.
    add_signature(
        &pool,
        w.owner,
        AddSignature {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            signature_id: "BBB-222".into(),
            group: SignatureGroup::Data,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        pasted_at(&pool, w.owner, w.map_id, placement).await,
        Some(first)
    );

    // Pasting the same scan again still counts: the list was checked against it.
    paste_one(&pool, w.owner, w.map_id, "AAA-111").await;
    let second = pasted_at(&pool, w.owner, w.map_id, placement)
        .await
        .unwrap();
    assert!(second > first);

    undo(&pool, w.owner, w.map_id).await;
    assert_eq!(
        pasted_at(&pool, w.owner, w.map_id, placement).await,
        Some(first)
    );
    undo(&pool, w.owner, w.map_id).await; // the hand edit
    undo(&pool, w.owner, w.map_id).await; // the first paste
    assert_eq!(pasted_at(&pool, w.owner, w.map_id, placement).await, None);
}

#[sqlx::test]
async fn the_paste_time_comes_back_with_a_removed_system(pool: PgPool) {
    let w = world(&pool).await;
    let placement = place(&pool, w.owner, w.map_id, SYS_A).await;
    paste_one(&pool, w.owner, w.map_id, "AAA-111").await;
    let pasted = pasted_at(&pool, w.owner, w.map_id, placement).await;
    assert!(pasted.is_some());

    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: placement,
        },
    )
    .await
    .unwrap();
    undo(&pool, w.owner, w.map_id).await;

    assert_eq!(pasted_at(&pool, w.owner, w.map_id, placement).await, pasted);
}
