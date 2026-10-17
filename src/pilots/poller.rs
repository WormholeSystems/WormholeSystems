//! Two-tier character-status polling, see
//! [processes.md](../../docs/processes.md#character-status-polling).
//!
//! Tier 1 polls online state for the characters of active users every 60s; tier 2 polls
//! location and ship for the online ones every 5s. Both are bounded by a [`Semaphore`] so we
//! stay within ESI's error limit and fit each tier's time budget. A poll that observes an
//! actual change pings its user's private channel so their UI refetches.
//!
//! Each tier awaits its whole batch before the next tick, so one poll that never returns
//! would stop that tier for the life of the process. Every poll runs under
//! [`POLL_TIMEOUT`] for that reason, and failures are counted and logged once per tick:
//! as a warning, since ESI has bad minutes and the next tick retries, and as an error only
//! once a tier has failed on every tick for [`FAILING_FOR`].
//!
//! Tokens come from a [`TokenCache`], so a poll touches the database only to write what
//! actually changed. Most polls see nothing new and write nothing.

use std::collections::HashSet;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sqlx::PgPool;
use tokio::sync::{Semaphore, SemaphorePermit};
use tokio::task::JoinSet;
use tokio::time::timeout;

use crate::background::{Streak, spawn_every};
use crate::db::PgTokenStore;
use crate::esi::scopes::Scope;
use crate::esi::{EsiClient, EsiError, Result as EsiResult, Sso, Token, TokenCache, TokenStore};
use crate::realtime::{UserEvent, UserHub};
use crate::server_status::ServerWatch;

/// Max ESI requests in flight per tick, the one tuning knob. Raise to fit more characters
/// in the 5s tier-2 budget; lower to stay further under ESI's error limit.
const CONCURRENCY: usize = 32;

/// The longest one character's poll may take, token refresh included, before it is dropped
/// and counted as failed. The next tick retries it.
const POLL_TIMEOUT: Duration = Duration::from_secs(20);

/// How long a tier may fail on every tick before it is an error rather than a warning.
const FAILING_FOR: Duration = Duration::from_secs(15 * 60);

/// A poll's outcome. A missing scope is not a failure: that field is simply not tracked.
type Poll = Result<(), String>;

/// A character due for polling, with the user to notify.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Due {
    pub character_id: i64,
    pub user_id: i64,
}

/// What both tiers share.
struct Poller {
    pool: PgPool,
    sso: Arc<Sso>,
    esi: EsiClient,
    users: UserHub,
    store: PgTokenStore,
    tokens: TokenCache,
    /// The poller's share of the pool. [`CONCURRENCY`] polls can be in flight and each
    /// ends in a write, while the API, websockets and killmail ingest draw on the same
    /// connections; without a cap of its own a busy tick could take all of them.
    db: Semaphore,
    tier_one_failing: Streak,
    tier_two_failing: Streak,
}

impl Poller {
    /// Wait for one of the poller's connections. Held across one statement or one short
    /// run of them, never while waiting on ESI, and never two at once.
    async fn db(&self) -> SemaphorePermit<'_> {
        self.db.acquire().await.expect("semaphore open")
    }

    /// The character's token, loaded once per poll and checked for each field's scope.
    /// `None` when EVE has refused it: that is said once where it is found out, and the
    /// character drops out of the polled set until it logs in again.
    async fn token(&self, character_id: i64, wanted: &[Scope]) -> Result<Option<Token>, String> {
        match self
            .tokens
            .token(&self.sso, &Throttled(self), character_id, wanted)
            .await
        {
            Ok(token) => Ok(Some(token)),
            Err(EsiError::TokenRejected) => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }
}

/// The token store, inside the poller's share of the pool.
struct Throttled<'a>(&'a Poller);

impl TokenStore for Throttled<'_> {
    async fn load(&self, character_id: i64) -> EsiResult<Option<Token>> {
        let _db = self.0.db().await;
        self.0.store.load(character_id).await
    }

    async fn save(&self, character_id: i64, token: &Token) -> EsiResult<()> {
        let _db = self.0.db().await;
        self.0.store.save(character_id, token).await
    }

    async fn reject(&self, character_id: i64) -> EsiResult<bool> {
        let _db = self.0.db().await;
        self.0.store.reject(character_id).await
    }
}

/// Spawn the polling loops. Returns immediately; the loops run for the process lifetime.
pub fn start(pool: PgPool, sso: Arc<Sso>, esi: EsiClient, users: UserHub, server: ServerWatch) {
    // Half the pool, leaving the rest to requests and the other background work.
    let db = (pool.options().get_max_connections() as usize / 2).max(1);
    let poller = Arc::new(Poller {
        store: PgTokenStore::new(pool.clone()),
        pool,
        sso,
        esi,
        users,
        tokens: TokenCache::new(),
        db: Semaphore::new(db),
        tier_one_failing: Streak::new(FAILING_FOR),
        tier_two_failing: Streak::new(FAILING_FOR),
    });
    let (one, one_server) = (poller.clone(), server.clone());
    spawn_every("tracking tier-1", Duration::from_secs(60), move || {
        tier_one(one.clone(), one_server.clone())
    });
    spawn_every("tracking tier-2", Duration::from_secs(5), move || {
        tier_two(poller.clone(), server.clone())
    });
}

/// Tier 1: online state, every 60s, for every character of an active user.
async fn tier_one(poller: Arc<Poller>, server: ServerWatch) -> Result<(), sqlx::Error> {
    // First let go of whoever dropped out of polling, server up or not.
    let gone = {
        let _db = poller.db().await;
        mark_unpolled_offline(&poller.pool).await?
    };
    let mut users = HashSet::new();
    for d in &gone {
        poller.users.publish(
            d.user_id,
            UserEvent::CharacterStatusChanged {
                character_id: d.character_id,
            },
        );
        users.insert(d.user_id);
    }
    for user_id in users {
        announce_presence(&poller, user_id).await;
    }

    // Nobody can be online while the server is not, and every call would fail against the
    // error limit. See [`crate::server_status`].
    if !server.should_poll() {
        skipped_offline("tier-1");
        return Ok(());
    }
    let due = active_characters(&poller.pool, false).await?;
    // Everyone tier 2 polls is in this set too, so this is where tokens for characters that
    // dropped out of polling are let go.
    let ids: HashSet<i64> = due.iter().map(|d| d.character_id).collect();
    poller.tokens.retain(|id| ids.contains(&id));
    run_polls("tier-1", &poller.tier_one_failing, &due, |d| {
        poll_online(poller.clone(), d)
    })
    .await;
    Ok(())
}

/// Tier 2: location + ship, every 5s, for currently-online characters of active users.
async fn tier_two(poller: Arc<Poller>, server: ServerWatch) -> Result<(), sqlx::Error> {
    if !server.should_poll() {
        skipped_offline("tier-2");
        return Ok(());
    }
    let due = active_characters(&poller.pool, true).await?;
    run_polls("tier-2", &poller.tier_two_failing, &due, |d| {
        poll_location_ship(poller.clone(), d)
    })
    .await;
    Ok(())
}

/// A tick that did nothing because Tranquility is not up.
fn skipped_offline(tier: &'static str) {
    metrics::counter!("ws_tracking_ticks_skipped_total", "tier" => tier, "reason" => "server_offline")
        .increment(1);
}

/// Characters whose user has been active within the last 5 minutes and that hold a token EVE
/// has not refused. With `online_only`, also restricted to those currently flagged online
/// (the tier-2 set).
async fn active_characters(pool: &PgPool, online_only: bool) -> Result<Vec<Due>, sqlx::Error> {
    let rows = if online_only {
        sqlx::query!(
            r#"select c.id, c.user_id as "user_id!"
             from characters c
             join users u on u.id = c.user_id
             join character_status s on s.character_id = c.id
             where u.last_active_at > now() - interval '5 minutes' and s.online
               and exists (select 1 from esi_tokens t
                           where t.character_id = c.id and t.rejected_at is null)"#
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| Due {
            character_id: r.id,
            user_id: r.user_id,
        })
        .collect()
    } else {
        sqlx::query!(
            r#"select c.id, c.user_id as "user_id!"
             from characters c
             join users u on u.id = c.user_id
             where u.last_active_at > now() - interval '5 minutes'
               and exists (select 1 from esi_tokens t
                           where t.character_id = c.id and t.rejected_at is null)"#
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| Due {
            character_id: r.id,
            user_id: r.user_id,
        })
        .collect()
    };
    Ok(rows)
}

/// Mark offline every character still flagged online that the poller no longer asks about:
/// its user has been idle past the 5 minute window, or EVE refused its token. Nobody would
/// ever see them log off otherwise, and they would stay on every map where they last were.
/// Pilots are on a map only while their user has the app open, as in legacy.
///
/// Returns the characters it flipped, with their users, so they can be told.
pub async fn mark_unpolled_offline(pool: &PgPool) -> Result<Vec<Due>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"update character_status s
           set online = false, last_online_at = now(), updated_at = now()
           from characters c
           join users u on u.id = c.user_id
           where c.id = s.character_id and s.online
             and (u.last_active_at is null
                  or u.last_active_at <= now() - interval '5 minutes'
                  or not exists (select 1 from esi_tokens t
                                 where t.character_id = c.id and t.rejected_at is null))
           returning s.character_id, c.user_id as "user_id!""#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| Due {
            character_id: r.character_id,
            user_id: r.user_id,
        })
        .collect())
}

/// Run `f` over every item with at most `concurrency` in flight; await the whole batch.
pub(crate) async fn run_bounded<T, F, Fut>(items: &[T], concurrency: usize, f: F)
where
    T: Copy + Send + 'static,
    F: Fn(T) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let permits = Arc::new(Semaphore::new(concurrency));
    let mut set = JoinSet::new();
    for &item in items {
        // Acquiring before spawning is the backpressure: never more than CONCURRENCY tasks.
        let permit = permits
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore open");
        let fut = f(item);
        set.spawn(async move {
            let _permit = permit;
            fut.await;
        });
    }
    while set.join_next().await.is_some() {}
}

/// [`run_bounded`] for one tier's polls: each under [`POLL_TIMEOUT`], with a single line
/// logged for the tick when any failed, and an error once `streak` says the tier is stuck.
/// The message stays the same from tick to tick, so the reports group as one issue.
async fn run_polls<F, Fut>(tier: &'static str, streak: &Streak, due: &[Due], f: F)
where
    F: Fn(Due) -> Fut,
    Fut: Future<Output = Poll> + Send + 'static,
{
    // How many failed, and the first one's reason as a sample.
    let failed = Arc::new(Mutex::new((0usize, None::<String>)));
    run_bounded(due, CONCURRENCY, |d| {
        let poll = timeout(POLL_TIMEOUT, f(d));
        let failed = failed.clone();
        async move {
            let err = match poll.await {
                Ok(Ok(())) => return,
                Ok(Err(err)) => err,
                Err(_) => format!("timed out after {}s", POLL_TIMEOUT.as_secs()),
            };
            let mut failed = failed.lock().expect("poll failures lock");
            failed.0 += 1;
            failed
                .1
                .get_or_insert_with(|| format!("character {}: {err}", d.character_id));
        }
    })
    .await;
    let (count, sample) = std::mem::take(&mut *failed.lock().expect("poll failures lock"));
    let polls = |outcome: &'static str, n: usize| {
        metrics::counter!("ws_tracking_polls_total", "tier" => tier, "outcome" => outcome)
            .increment(n as u64);
    };
    polls("ok", due.len() - count);
    polls("failed", count);
    let Some(sample) = sample else {
        streak.record(false, Instant::now());
        return;
    };
    if streak.record(true, Instant::now()) {
        tracing::error!(
            failed = count,
            polls = due.len(),
            sample,
            "tracking {tier}: polls have failed on every tick for {} minutes",
            FAILING_FOR.as_secs() / 60
        );
    } else {
        tracing::warn!(
            failed = count,
            polls = due.len(),
            sample,
            "tracking {tier}: polls failed"
        );
    }
}

/// Poll one character's online state (tier 1). A failure skips the character; the next tick
/// retries.
async fn poll_online(poller: Arc<Poller>, due: Due) -> Poll {
    let Some(token) = poller.token(due.character_id, &[Scope::ReadOnline]).await? else {
        return Ok(());
    };
    if !token.has_scope(Scope::ReadOnline) {
        return Ok(());
    }
    let status = poller
        .esi
        .character_online(&token.access_token, due.character_id)
        .await
        .map_err(|err| err.to_string())?;
    // Written only on a transition, which is also when the user is pinged: a row comes
    // back only when one was inserted or actually changed. `last_online_at` moves on both
    // edges, so once offline it says when they were last seen online, to within a tick.
    let changed = {
        let _db = poller.db().await;
        sqlx::query_scalar!(
            "insert into character_status (character_id, online, last_online_at)
             values ($1, $2, case when $2 then now() else null end)
             on conflict (character_id) do update set
                 online = excluded.online,
                 last_online_at = now(),
                 updated_at = now()
             where character_status.online is distinct from excluded.online
             returning character_id",
            due.character_id,
            status.online,
        )
        .fetch_optional(&poller.pool)
        .await
        .map_err(|err| err.to_string())?
        .is_some()
    };
    if changed {
        poller.users.publish(
            due.user_id,
            UserEvent::CharacterStatusChanged {
                character_id: due.character_id,
            },
        );
        announce_presence(&poller, due.user_id).await;
    }
    Ok(())
}

/// Poll one character's location and ship (tier 2). The two use independent scopes, so a
/// missing one skips just that field, and one failing does not stop the other. Docking is
/// left null until we cache those entities.
async fn poll_location_ship(poller: Arc<Poller>, due: Due) -> Poll {
    let id = due.character_id;
    let Some(token) = poller
        .token(id, &[Scope::ReadLocation, Scope::ReadShipType])
        .await?
    else {
        return Ok(());
    };
    let location = if token.has_scope(Scope::ReadLocation) {
        poll_location(&poller, &token.access_token, id).await
    } else {
        Ok(false)
    };
    let ship = if token.has_scope(Scope::ReadShipType) {
        poll_ship(&poller, &token.access_token, id).await
    } else {
        Ok(false)
    };

    if matches!(location, Ok(true)) || matches!(ship, Ok(true)) {
        poller.users.publish(
            due.user_id,
            UserEvent::CharacterStatusChanged { character_id: id },
        );
        announce_presence(&poller, due.user_id).await;
    }
    location.and(ship).map(|_| ())
}

/// The location half of tier 2. `Ok(true)` when it moved.
async fn poll_location(poller: &Poller, token: &str, id: i64) -> Result<bool, String> {
    let location = poller
        .esi
        .character_location(token, id)
        .await
        .map_err(|err| err.to_string())?;
    // As in tier 1, only a change is written, and a row comes back only for one. No row
    // also covers a character with no status yet: tier 1 creates it.
    let moved = {
        let _db = poller.db().await;
        sqlx::query!(
            r#"with prev as (
                     select solar_system_id from character_status where character_id = $1
                 )
                 update character_status
                 set solar_system_id = $2, station_id = null, structure_id = null,
                     updated_at = now()
                 where character_id = $1
                   and (solar_system_id is distinct from $2
                        or station_id is not null or structure_id is not null)
                 returning (select solar_system_id from prev) as "prev_solar_system_id?""#,
            id,
            location.solar_system_id,
        )
        .fetch_optional(&poller.pool)
        .await
        .map_err(|err| err.to_string())?
    };
    let Some(moved) = moved else {
        return Ok(false);
    };

    // A system change is a potential wormhole transit: jump capture must never
    // break polling, so failures are logged and dropped.
    if let Some(prev) = moved.prev_solar_system_id
        && prev != location.solar_system_id
    {
        let _db = poller.db().await;
        if let Err(err) =
            crate::maps::jumps::record_transit(&poller.pool, id, prev, location.solar_system_id)
                .await
        {
            tracing::error!("jump capture failed for character {id}: {err}");
        }
    }
    Ok(true)
}

/// The ship half of tier 2. `Ok(true)` when the hull or its name changed.
async fn poll_ship(poller: &Poller, token: &str, id: i64) -> Result<bool, String> {
    let ship = poller
        .esi
        .character_ship(token, id)
        .await
        .map_err(|err| err.to_string())?;
    let _db = poller.db().await;
    let Some(row) = sqlx::query!(
        // The stamp moves only when the hull does, so it answers "how long have they
        // been in this ship" rather than "when did we last poll".
        r#"with prev as (
                 select ship_type_id, ship_name from character_status where character_id = $1
             )
             update character_status
             set ship_type_id = $2, ship_name = $3, ship_item_id = $4,
                 ship_updated_at = case
                     when ship_item_id is distinct from $4 then now()
                     else ship_updated_at
                 end,
                 updated_at = now()
             where character_id = $1
               and (ship_type_id, ship_name, ship_item_id)
                   is distinct from ($2::bigint, $3::text, $4::bigint)
             returning (select ship_type_id from prev) as "prev_ship_type_id?",
                       (select ship_name from prev) as "prev_ship_name?""#,
        id,
        ship.ship_type_id,
        ship.ship_name,
        ship.ship_item_id,
    )
    .fetch_optional(&poller.pool)
    .await
    .map_err(|err| err.to_string())?
    else {
        return Ok(false);
    };
    Ok(row.prev_ship_type_id != Some(ship.ship_type_id)
        || row.prev_ship_name.as_deref() != Some(ship.ship_name.as_str()))
}

/// Tell every map this user shares their position with that its pilot list moved. The
/// per-user ping above only reaches the pilot's own tabs.
async fn announce_presence(poller: &Poller, user_id: i64) {
    let map_ids = {
        let _db = poller.db().await;
        sqlx::query_scalar!(
            "select map_id from map_user_settings where user_id = $1 and tracking_allowed",
            user_id,
        )
        .fetch_all(&poller.pool)
        .await
    };
    let Ok(map_ids) = map_ids else {
        return;
    };
    for map_id in map_ids {
        crate::realtime::hub().publish(crate::realtime::MapEvent::CharactersChanged { map_id });
    }
}
