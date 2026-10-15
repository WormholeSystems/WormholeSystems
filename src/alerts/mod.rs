//! Discord alerts: rules a map watches for, and the messages they send.
//!
//! Everything here is best-effort and off the critical path. Evaluating an alert must
//! never hold up ingesting a killmail or placing a system, and a Discord outage must cost
//! a message rather than a map change.

pub mod delivery;
pub mod filters;
pub mod jump_range;
pub mod killmail;
pub mod place;
pub mod proximity;
pub mod ships;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sqlx::PgPool;
use tokio::sync::{Semaphore, mpsc};

/// How many messages may wait for Discord. Past this, Discord is down or throttling us,
/// and queueing more would only deliver stale news later.
const OUTBOX_CAPACITY: usize = 256;

/// Messages in flight at once, so one webhook that takes its full timeout to fail does not
/// hold up every other map's alerts behind it.
const SENDERS: usize = 4;

/// How long the cached alert list is trusted without a known change. Every write through
/// this crate invalidates it at once; this is the backstop for one that does not.
const CACHE_TTL: Duration = Duration::from_secs(60);

/// How far back a lagged map-event subscriber looks to catch up on what it missed: from
/// the last event it took, plus some slack, but never more than the cap.
const LAG_REPLAY_SLACK: Duration = Duration::from_secs(30);
const LAG_REPLAY_CAP: Duration = Duration::from_secs(10 * 60);

/// Bumped by every change to an alert or its destination, so the cache knows to reload.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Mark the cached alerts as stale. Called by [`log`], which every alert write goes
/// through, so a new alert is live on the next kill rather than the next minute.
pub fn changed() {
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

/// One message, evaluated and claimed, waiting to be sent.
pub struct Outgoing {
    pub alert: Alert,
    pub key: String,
    pub embed: delivery::Embed,
}

struct Snapshot {
    loaded_at: Instant,
    generation: u64,
    alerts: Arc<Vec<Alert>>,
}

/// The stargate graph, the HTTP client and the active alerts, held for the life of the
/// process so none of them is rebuilt per killmail.
pub struct Runtime {
    universe: proximity::Universe,
    http: reqwest::Client,
    /// Absent unless a bot is configured; only channel and direct-message delivery need it.
    bot_token: Option<String>,
    /// Where the app lives, so a message can link back to the map. Discord refuses an
    /// embed whose link is not absolute.
    public_url: String,
    cache: Mutex<Option<Snapshot>>,
    /// The delivery worker. Evaluation only ever hands it messages, so a slow Discord
    /// holds up messages and nothing else.
    outbox: mpsc::Sender<Outgoing>,
}

impl Runtime {
    /// Load the graph and start the delivery worker.
    pub async fn load(
        pool: &PgPool,
        bot_token: Option<String>,
        public_url: String,
    ) -> sqlx::Result<Runtime> {
        let http = crate::user_agent::client();
        let (outbox, inbox) = mpsc::channel(OUTBOX_CAPACITY);
        tokio::spawn(deliver_loop(
            pool.clone(),
            http.clone(),
            bot_token.clone(),
            inbox,
        ));
        Ok(Runtime {
            bot_token,
            public_url,
            universe: proximity::Universe::load(pool).await?,
            http,
            cache: Mutex::new(None),
            outbox,
        })
    }

    fn token(&self) -> Option<&str> {
        self.bot_token.as_deref()
    }

    /// Every active alert, from memory unless something changed or the copy is old.
    async fn alerts(&self, pool: &PgPool) -> Arc<Vec<Alert>> {
        let generation = GENERATION.load(Ordering::Relaxed);
        if let Some(snapshot) = self.cache.lock().expect("alert cache poisoned").as_ref()
            && snapshot.generation == generation
            && snapshot.loaded_at.elapsed() < CACHE_TTL
        {
            return snapshot.alerts.clone();
        }
        // Loaded outside the lock: two callers racing here both load, which is harmless.
        let alerts = match fetch_active(pool, None).await {
            Ok(alerts) => Arc::new(alerts),
            Err(err) => {
                tracing::error!("alerts: could not load the active alerts: {err}");
                return Arc::default();
            }
        };
        *self.cache.lock().expect("alert cache poisoned") = Some(Snapshot {
            loaded_at: Instant::now(),
            generation,
            alerts: alerts.clone(),
        });
        alerts
    }

    /// The active alerts of a kind, optionally on one map only.
    async fn of_kind(&self, pool: &PgPool, kind: AlertKind, map_id: Option<i64>) -> Vec<Alert> {
        self.alerts(pool)
            .await
            .iter()
            .filter(|a| a.kind == kind && map_id.is_none_or(|m| a.map_id == m))
            .cloned()
            .collect()
    }

    /// Offer a batch of kills to every alert watching for one. Each map's chain is read
    /// once per batch, however many kills land near it.
    pub async fn killmails(&self, pool: &PgPool, kills: &[killmail::Kill]) -> Vec<Outgoing> {
        let alerts = self.of_kind(pool, AlertKind::Killmail, None).await;
        if alerts.is_empty() {
            return Vec::new();
        }
        let mut chains = HashMap::new();
        let mut out = Vec::new();
        for kill in kills {
            out.extend(killmail::evaluate(pool, &self.universe, &alerts, kill, &mut chains).await);
        }
        out
    }

    /// Re-evaluate a map's alerts after its shape changed. A new system can put a target
    /// within gate range, within jump range, or neither, so both kinds fire from here.
    pub async fn placed(
        &self,
        pool: &PgPool,
        map_id: i64,
        map_solar_system_id: i64,
    ) -> Vec<Outgoing> {
        let mut out = Vec::new();
        let proximity = self.of_kind(pool, AlertKind::Proximity, Some(map_id)).await;
        if !proximity.is_empty() {
            out.extend(
                place::evaluate(
                    pool,
                    &self.universe,
                    &proximity,
                    map_id,
                    place::Occasion::Placed(map_solar_system_id),
                    &self.public_url,
                )
                .await,
            );
        }
        let jump_range = self.of_kind(pool, AlertKind::JumpRange, Some(map_id)).await;
        if !jump_range.is_empty() {
            out.extend(jump_range::evaluate(pool, &jump_range, map_solar_system_id).await);
        }
        out
    }

    /// Re-evaluate a map's proximity alerts after a wormhole joined two of its systems. The
    /// edge can complete a route from an alert's starting point even though nothing was
    /// placed, so both ends count as if they had just landed.
    pub async fn connected(&self, pool: &PgPool, map_id: i64, connection_id: i64) -> Vec<Outgoing> {
        let proximity = self.of_kind(pool, AlertKind::Proximity, Some(map_id)).await;
        if proximity.is_empty() {
            return Vec::new();
        }
        let Ok(Some(ends)) = sqlx::query!(
            "select from_system, to_system from map_connections
             where id = $1 and map_id = $2 and type = 'wormhole'",
            connection_id,
            map_id,
        )
        .fetch_optional(pool)
        .await
        else {
            return Vec::new();
        };
        place::evaluate(
            pool,
            &self.universe,
            &proximity,
            map_id,
            place::Occasion::Connected(ends.from_system, ends.to_system),
            &self.public_url,
        )
        .await
    }

    /// Send one message now, waiting for Discord.
    pub async fn send(&self, pool: &PgPool, out: Outgoing) {
        fire(
            pool,
            &self.http,
            self.token(),
            &out.alert,
            &out.key,
            out.embed,
        )
        .await;
    }

    /// Hand messages to the delivery worker without waiting for Discord.
    ///
    /// A full queue drops the message rather than waiting: whatever is evaluating must not
    /// stall behind Discord, and by the time a backlog that deep drained, the news would be
    /// stale. The claim is handed back, so the next occasion is free to try again.
    pub async fn enqueue(&self, pool: &PgPool, messages: Vec<Outgoing>) {
        for out in messages {
            if let Err(err) = self.outbox.try_send(out) {
                let out = match err {
                    mpsc::error::TrySendError::Full(out) => {
                        tracing::warn!(
                            "alerts: delivery queue full, dropping alert {} ({})",
                            out.alert.id,
                            out.key
                        );
                        out
                    }
                    mpsc::error::TrySendError::Closed(out) => out,
                };
                unclaim(pool, out.alert.id, &out.key).await;
            }
        }
    }

    /// Catch up after the map-event subscriber fell behind. The events themselves are
    /// gone, so every placement and wormhole made in the gap, on a map with an alert that
    /// cares, is evaluated again. The claims make that harmless for the ones that were
    /// handled before the gap.
    async fn replay(&self, pool: &PgPool, since: Duration) -> Vec<Outgoing> {
        let alerts = self.alerts(pool).await;
        let mut maps: Vec<i64> = alerts
            .iter()
            .filter(|a| matches!(a.kind, AlertKind::Proximity | AlertKind::JumpRange))
            .map(|a| a.map_id)
            .collect();
        maps.sort_unstable();
        maps.dedup();
        if maps.is_empty() {
            return Vec::new();
        }
        let secs = since.min(LAG_REPLAY_CAP).as_secs_f64();
        let placed = sqlx::query!(
            "select map_id, id from map_solar_systems
             where map_id = any($1) and created_at > now() - make_interval(secs => $2)
             order by id",
            &maps,
            secs,
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        let connected = sqlx::query!(
            "select map_id, id from map_connections
             where map_id = any($1) and type = 'wormhole'
               and created_at > now() - make_interval(secs => $2)
             order by id",
            &maps,
            secs,
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        let mut out = Vec::new();
        for row in placed {
            out.extend(self.placed(pool, row.map_id, row.id).await);
        }
        for row in connected {
            out.extend(self.connected(pool, row.map_id, row.id).await);
        }
        out
    }
}

/// Send what evaluation queued, a few at a time.
async fn deliver_loop(
    pool: PgPool,
    http: reqwest::Client,
    bot_token: Option<String>,
    mut inbox: mpsc::Receiver<Outgoing>,
) {
    let senders = Arc::new(Semaphore::new(SENDERS));
    while let Some(out) = inbox.recv().await {
        let permit = senders
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore open");
        let (pool, http, bot_token) = (pool.clone(), http.clone(), bot_token.clone());
        tokio::spawn(async move {
            fire(
                &pool,
                &http,
                bot_token.as_deref(),
                &out.alert,
                &out.key,
                out.embed,
            )
            .await;
            drop(permit);
        });
    }
}

/// Watch every map for the changes alerts care about.
///
/// A subscriber rather than a hook in the command dispatcher, so nothing here can fail a
/// placement. It only evaluates, which is a few queries on a map with alerts and nothing
/// on one without, and leaves Discord to the delivery worker; that keeps it ahead of a big
/// paste. If it does fall behind, it replays the gap from the tables.
pub fn start(pool: PgPool, runtime: Arc<Runtime>) {
    start_on(pool, crate::realtime::hub().clone(), runtime);
}

/// [`start`] on a hub of the caller's. For tests: the process-wide hub carries every other
/// test's maps, whose ids collide with this one's across test databases.
#[doc(hidden)]
pub fn start_on(pool: PgPool, hub: crate::realtime::MapHub, runtime: Arc<Runtime>) {
    tokio::spawn(async move {
        let mut events = hub.subscribe_all();
        // When the last event came off the channel. What was missed was published after
        // it, though not necessarily after it was received: it may have waited behind
        // others. Hence the slack on the replay.
        let mut received = Instant::now();
        loop {
            let event = events.recv().await;
            let out = match event {
                Ok(crate::realtime::MapEvent::SystemAdded {
                    map_id,
                    map_solar_system_id,
                }) => {
                    received = Instant::now();
                    runtime.placed(&pool, map_id, map_solar_system_id).await
                }
                Ok(crate::realtime::MapEvent::ConnectionChanged {
                    map_id,
                    connection_id,
                }) => {
                    received = Instant::now();
                    runtime.connected(&pool, map_id, connection_id).await
                }
                Ok(_) => {
                    received = Instant::now();
                    Vec::new()
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
                    tracing::warn!("alerts: map events lagged, {missed} missed; replaying the gap");
                    let since = received.elapsed() + LAG_REPLAY_SLACK;
                    received = Instant::now();
                    runtime.replay(&pool, since).await
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
            };
            runtime.enqueue(&pool, out).await;
        }
    });
}

crate::maps::text_enum! {
    /// What an alert watches for.
    pub enum AlertKind {
        /// Something died within reach of the chain.
        Killmail => "killmail",
        /// A system came within reach of the chain.
        Proximity => "proximity",
        /// A system came within capital jump range of a k-space exit.
        JumpRange => "jump_range",
    }
}

crate::maps::text_enum! {
    /// Where the message goes.
    pub enum AlertDelivery {
        /// A channel webhook URL, which needs no bot and no permissions.
        Webhook => "webhook",
        /// A direct message to whoever created the alert.
        DiscordDm => "discord_dm",
        /// A channel the bot can post in.
        DiscordChannel => "discord_channel",
    }
}

impl AlertDelivery {
    /// Whether delivery needs the bot rather than a webhook URL.
    pub fn needs_bot(self) -> bool {
        matches!(
            self,
            AlertDelivery::DiscordDm | AlertDelivery::DiscordChannel
        )
    }
}

crate::maps::text_enum! {
    /// Who gets pinged when an alert fires.
    pub enum AlertMention {
        None => "none",
        /// Whoever created the alert, by their linked Discord account.
        Creator => "creator",
        /// The configured role.
        Role => "role",
        Everyone => "everyone",
    }
}

crate::maps::text_enum! {
    /// Why an alert stopped firing, so the settings page can say more than "off".
    pub enum DisabledReason {
        Manual => "manual",
        DiscordUnlinked => "discord_unlinked",
        AccessRevoked => "access_revoked",
        DestinationGone => "destination_gone",
        DeliveryFailed => "delivery_failed",
    }
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: i64,
    pub map_id: i64,
    pub created_by_user_id: Option<i64>,
    pub name: String,
    pub kind: AlertKind,
    pub delivery: AlertDelivery,
    /// Resolved from the named destination the alert points at.
    pub webhook_url: Option<String>,
    pub discord_guild_id: Option<String>,
    pub discord_channel_id: Option<String>,
    /// Resolved from the named role the alert points at.
    pub discord_role_id: Option<String>,
    pub mention: AlertMention,
    pub target_solar_system_id: Option<i64>,
    /// Proximity only: measure from here through the chain, rather than from whichever
    /// mapped system is nearest the target.
    pub origin_solar_system_id: Option<i64>,
    pub max_jumps: i32,
    /// Jump range only: which hull's range is being measured, and the pilot's JDC level.
    pub ship_type: Option<ships::JumpShip>,
    pub jdc_level: Option<i32>,
    pub filters: Vec<filters::Rule>,
    pub filter_match: filters::Match,
    pub is_active: bool,
}

/// Every active alert of a kind, across every map. Loaded whole rather than per map: a
/// killmail arrives without knowing which maps care, and there are tens of alerts in total.
pub async fn active(pool: &PgPool, kind: AlertKind) -> sqlx::Result<Vec<Alert>> {
    fetch_active(pool, Some(kind)).await
}

/// Every active alert, of one kind or of all of them.
async fn fetch_active(pool: &PgPool, kind: Option<AlertKind>) -> sqlx::Result<Vec<Alert>> {
    let rows = sqlx::query!(
        r#"select a.id, a.map_id, a.created_by_user_id, a.name, a.kind, a.delivery,
                  w.url as "webhook_url?",
                  a.discord_guild_id, a.discord_channel_id,
                  r.discord_role_id as "discord_role_id?",
                  a.mention, a.target_solar_system_id, a.origin_solar_system_id,
                  a.max_jumps, a.ship_type, a.jdc_level,
                  a.filters, a.filter_match, a.is_active
           from map_alerts a
           left join map_webhooks w on w.id = a.map_webhook_id
           left join map_webhook_roles r on r.id = a.map_webhook_role_id
           where ($1::text is null or a.kind = $1) and a.is_active
           order by a.id"#,
        kind.map(AlertKind::as_str),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            // A row that no longer decodes is skipped loudly rather than coerced into a
            // default: a corrupt `delivery` must not quietly become a webhook post, and
            // corrupt `filters` must not become "match every kill".
            let (Some(kind), Some(delivery), Some(mention), Some(filter_match), Ok(filters)) = (
                AlertKind::from_db(&row.kind),
                AlertDelivery::from_db(&row.delivery),
                AlertMention::from_db(&row.mention),
                filters::Match::from_db(&row.filter_match),
                serde_json::from_value(row.filters),
            ) else {
                tracing::warn!(
                    "alerts: skipping alert {} ({}): row does not decode",
                    row.id,
                    row.name
                );
                return None;
            };
            Some(Alert {
                id: row.id,
                map_id: row.map_id,
                created_by_user_id: row.created_by_user_id,
                name: row.name,
                kind,
                delivery,
                webhook_url: row.webhook_url,
                discord_guild_id: row.discord_guild_id,
                discord_channel_id: row.discord_channel_id,
                discord_role_id: row.discord_role_id,
                mention,
                target_solar_system_id: row.target_solar_system_id,
                origin_solar_system_id: row.origin_solar_system_id,
                max_jumps: row.max_jumps,
                ship_type: row.ship_type.as_deref().and_then(ships::JumpShip::from_db),
                jdc_level: row.jdc_level,
                filters,
                filter_match,
                is_active: row.is_active,
            })
        })
        .collect())
}

/// Why a message did not go out, which decides what becomes of the alert and the claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Undelivered {
    /// Worth another try on the next occasion.
    Retry,
    /// The destination is gone; the alert stops.
    Gone,
    /// Discord refused this message itself. Sending it again gets the same answer, so the
    /// occasion stays claimed, and the alert carries on for the next one.
    Rejected,
}

/// Deliver one alert, once: claim the dedup key, send, and mark it sent, or hand the key
/// back so a later run can retry. Only a gone destination disables the alert.
pub async fn fire(
    pool: &PgPool,
    http: &reqwest::Client,
    bot_token: Option<&str>,
    alert: &Alert,
    dedup_key: &str,
    embed: delivery::Embed,
) {
    match deliver(pool, http, bot_token, alert, embed).await {
        Ok(()) => sent(pool, alert.id, dedup_key).await,
        Err(Undelivered::Retry) => unclaim(pool, alert.id, dedup_key).await,
        Err(Undelivered::Gone) => {
            unclaim(pool, alert.id, dedup_key).await;
            disable(pool, alert, DisabledReason::DestinationGone, None).await;
        }
        Err(Undelivered::Rejected) => {}
    }
}

/// Send one alert's message, wherever it is meant to go.
pub async fn deliver(
    pool: &PgPool,
    http: &reqwest::Client,
    bot_token: Option<&str>,
    alert: &Alert,
    embed: delivery::Embed,
) -> Result<(), Undelivered> {
    // Checked at send time rather than hooked to every access change: access can be lost in
    // half a dozen ways, and this is the one place that must be right.
    if let Some(creator) = alert.created_by_user_id
        && !crate::maps::access::can_see(pool, alert.map_id, creator).await
    {
        disable(pool, alert, DisabledReason::AccessRevoked, None).await;
        return Err(Undelivered::Retry);
    }

    let creator_discord = match alert.created_by_user_id {
        Some(user_id) => crate::discord::account_for(pool, user_id)
            .await
            .map(|a| a.discord_user_id),
        None => None,
    };

    let message = match alert.mention {
        AlertMention::Role => match alert.discord_role_id.as_deref() {
            Some(role) => delivery::Message::new(embed).mention_role(role),
            None => delivery::Message::new(embed),
        },
        AlertMention::Everyone => delivery::Message::new(embed).mention_everyone(),
        AlertMention::Creator => match creator_discord.as_deref() {
            Some(user) => delivery::Message::new(embed).mention_user(user),
            // Asking to ping someone who has not linked is not a broken destination, just
            // a message without a ping.
            None => delivery::Message::new(embed),
        },
        AlertMention::None => delivery::Message::new(embed),
    };

    let result = match alert.delivery {
        AlertDelivery::Webhook => match alert.webhook_url.as_deref() {
            Some(url) => delivery::post_webhook(http, url, &message).await,
            None => return Err(Undelivered::Gone),
        },
        AlertDelivery::DiscordChannel => {
            match (bot_token, alert.discord_channel_id.as_deref()) {
                (Some(token), Some(channel)) => {
                    delivery::post_channel(http, token, channel, &message).await
                }
                // No bot configured is an operator problem, not a broken alert: leave it
                // active so it starts working when the token appears.
                (None, _) => return Err(Undelivered::Retry),
                (_, None) => return Err(Undelivered::Gone),
            }
        }
        AlertDelivery::DiscordDm => match (bot_token, creator_discord.as_deref()) {
            (Some(token), Some(user)) => delivery::post_dm(http, token, user, &message).await,
            (None, _) => return Err(Undelivered::Retry),
            (_, None) => {
                disable(pool, alert, DisabledReason::DiscordUnlinked, None).await;
                return Err(Undelivered::Retry);
            }
        },
    };

    match result {
        Ok(()) => Ok(()),
        Err(delivery::SendError::Gone) => Err(Undelivered::Gone),
        Err(delivery::SendError::Rejected(body)) => {
            tracing::warn!(
                "alerts: discord refused the message for alert {}: {body}",
                alert.id
            );
            log(
                pool,
                Some(alert.id),
                alert.map_id,
                None,
                "failed",
                Some(&format!("discord refused the message: {body}")),
            )
            .await;
            Err(Undelivered::Rejected)
        }
        Err(delivery::SendError::Failed(err)) => {
            log(
                pool,
                Some(alert.id),
                alert.map_id,
                None,
                "failed",
                Some(&err),
            )
            .await;
            Err(Undelivered::Retry)
        }
    }
}

/// Claim the right to deliver this alert for this occasion; `false` means somebody already
/// has. Claimed before the message goes out, so a crash between the two costs a message
/// rather than sending it twice.
pub async fn claim(pool: &PgPool, alert_id: i64, dedup_key: &str) -> bool {
    sqlx::query!(
        "insert into map_alert_deliveries (map_alert_id, dedup_key)
         values ($1, $2) on conflict do nothing",
        alert_id,
        dedup_key,
    )
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
    .unwrap_or(false)
}

/// Mark a claimed delivery as actually sent, and stamp the alert.
pub async fn sent(pool: &PgPool, alert_id: i64, dedup_key: &str) {
    let _ = sqlx::query!(
        "update map_alert_deliveries set delivered_at = now()
         where map_alert_id = $1 and dedup_key = $2",
        alert_id,
        dedup_key,
    )
    .execute(pool)
    .await;
    let _ = sqlx::query!(
        "update map_alerts set last_fired_at = now() where id = $1",
        alert_id,
    )
    .execute(pool)
    .await;
}

/// Give up on a claimed delivery, so the next occasion can try again.
pub async fn unclaim(pool: &PgPool, alert_id: i64, dedup_key: &str) {
    let _ = sqlx::query!(
        "delete from map_alert_deliveries
         where map_alert_id = $1 and dedup_key = $2 and delivered_at is null",
        alert_id,
        dedup_key,
    )
    .execute(pool)
    .await;
}

/// Record something that happened to an alert.
pub async fn log(
    pool: &PgPool,
    alert_id: Option<i64>,
    map_id: i64,
    actor: Option<i64>,
    kind: &str,
    detail: Option<&str>,
) {
    // Every write to an alert or its destination is logged, so this is where the cached
    // alerts learn they are stale. A delivery failure reloads them needlessly; that is cheap.
    changed();
    let _ = sqlx::query!(
        "insert into map_alert_events (map_alert_id, map_id, actor_user_id, kind, detail)
         values ($1, $2, $3, $4, $5)",
        alert_id,
        map_id,
        actor,
        kind,
        detail,
    )
    .execute(pool)
    .await;
}

/// Turn an alert off, saying why.
pub async fn disable(pool: &PgPool, alert: &Alert, reason: DisabledReason, detail: Option<&str>) {
    let _ = sqlx::query!(
        "update map_alerts
         set is_active = false, disabled_at = now(), disabled_reason = $2, updated_at = now()
         where id = $1",
        alert.id,
        reason.as_str(),
    )
    .execute(pool)
    .await;
    log(
        pool,
        Some(alert.id),
        alert.map_id,
        None,
        "disabled",
        detail.or(Some(reason.as_str())),
    )
    .await;
}
