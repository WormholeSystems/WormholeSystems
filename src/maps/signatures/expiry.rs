//! The sweep that purges signatures nobody has seen for days.

use sqlx::PgPool;

use crate::maps::error::Result;
use crate::maps::{MapEvent, MapHub};

/// Purge stale signatures (legacy expiry): unlinked wormhole sigs older than 3 days, other
/// sigs untouched for 7 days (presence in a paste refreshes `updated_at`, keeping live
/// sites alive). Linked sigs are never expired: they represent a mapped connection.
/// Returns the affected `(map_id, solar_system_id)` pairs.
pub async fn expire_signatures(pool: &PgPool) -> Result<Vec<(i64, i64)>> {
    let rows = sqlx::query!(
        r#"delete from signatures
           where connection_id is null
             and (("group" = 'wormhole' and created_at < now() - interval '3 days')
                  or ("group" <> 'wormhole' and updated_at < now() - interval '7 days'))
           returning map_id, solar_system_id"#,
    )
    .fetch_all(pool)
    .await?;
    let mut pairs: Vec<(i64, i64)> = rows
        .into_iter()
        .map(|r| (r.map_id, r.solar_system_id))
        .collect();
    pairs.sort_unstable();
    pairs.dedup();
    Ok(pairs)
}

/// Publishes a `SignatureChanged` per affected system so open maps refresh. The cutoffs are
/// days, so the tick cadence is uncritical.
pub fn start_expiry(pool: PgPool, hub: MapHub) {
    crate::background::spawn_every(
        "signature expiry",
        std::time::Duration::from_secs(6 * 60 * 60),
        move || {
            let (pool, hub) = (pool.clone(), hub.clone());
            async move {
                for (map_id, solar_system_id) in expire_signatures(&pool).await? {
                    hub.publish(MapEvent::SignatureChanged {
                        map_id,
                        solar_system_id,
                    });
                }
                Ok::<_, crate::maps::MapError>(())
            }
        },
    );
}
