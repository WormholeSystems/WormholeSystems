//! The alert store: the settings page's CRUD over `map_alerts`, with the row decoding
//! in exactly one place. Alerts do not go through the command journal: they are
//! configuration, not map history.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::alerts::ships::JumpShip;
use crate::alerts::{AlertDelivery, AlertKind, AlertMention, filters};

use super::error::{MapError, Result};

/// One alert as the settings page shows it.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapAlert {
    pub id: i64,
    pub map_id: i64,
    pub name: String,
    pub kind: AlertKind,
    pub delivery: AlertDelivery,
    #[ts(optional)]
    pub map_webhook_id: Option<i64>,
    #[ts(optional)]
    pub webhook_name: Option<String>,
    #[ts(optional)]
    pub discord_channel_id: Option<String>,
    #[ts(optional)]
    pub map_webhook_role_id: Option<i64>,
    #[ts(optional)]
    pub role_name: Option<String>,
    pub mention: AlertMention,
    #[ts(optional)]
    pub target_solar_system_id: Option<i64>,
    #[ts(optional)]
    pub target_system_name: Option<String>,
    #[ts(optional)]
    pub origin_solar_system_id: Option<i64>,
    #[ts(optional)]
    pub origin_system_name: Option<String>,
    pub max_jumps: i32,
    #[ts(optional)]
    pub ship_type: Option<JumpShip>,
    #[ts(optional)]
    pub jdc_level: Option<i32>,
    pub filters: Vec<filters::Rule>,
    pub filter_match: filters::Match,
    pub is_active: bool,
    #[ts(optional)]
    pub disabled_reason: Option<String>,
    #[ts(optional)]
    pub last_fired_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SaveAlert {
    pub name: String,
    pub kind: AlertKind,
    pub delivery: AlertDelivery,
    /// Which registered destination to post to.
    #[serde(default)]
    #[ts(optional)]
    pub map_webhook_id: Option<i64>,
    /// Where a bot channel alert posts. Never read from a request: only the Discord
    /// command sets these, from the channel Discord says it was run in. Over HTTP a channel
    /// alert keeps the channel it has (see [`keep_bot_destination`]).
    #[serde(skip)]
    pub discord_guild_id: Option<String>,
    #[serde(skip)]
    pub discord_channel_id: Option<String>,
    /// Which registered role to ping.
    #[serde(default)]
    #[ts(optional)]
    pub map_webhook_role_id: Option<i64>,
    pub mention: AlertMention,
    #[serde(default)]
    #[ts(optional)]
    pub target_solar_system_id: Option<i64>,
    /// Proximity only: measure from here through the chain instead of from the nearest
    /// mapped system.
    #[serde(default)]
    #[ts(optional)]
    pub origin_solar_system_id: Option<i64>,
    pub max_jumps: i32,
    #[serde(default)]
    #[ts(optional)]
    pub ship_type: Option<JumpShip>,
    #[serde(default)]
    #[ts(optional)]
    pub jdc_level: Option<i32>,
    #[serde(default)]
    pub filters: Vec<filters::Rule>,
    pub filter_match: filters::Match,
}

struct AlertRow {
    id: i64,
    map_id: i64,
    name: String,
    kind: String,
    delivery: String,
    map_webhook_id: Option<i64>,
    webhook_name: Option<String>,
    discord_channel_id: Option<String>,
    map_webhook_role_id: Option<i64>,
    role_name: Option<String>,
    mention: String,
    target_solar_system_id: Option<i64>,
    target_system_name: Option<String>,
    origin_solar_system_id: Option<i64>,
    origin_system_name: Option<String>,
    max_jumps: i32,
    ship_type: Option<String>,
    jdc_level: Option<i32>,
    filters: serde_json::Value,
    filter_match: String,
    is_active: bool,
    disabled_reason: Option<String>,
    last_fired_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl AlertRow {
    /// Same rule as `alerts::active`: a row that no longer decodes is skipped loudly,
    /// never shown with made-up settings the next save would write back.
    fn decode(self) -> Option<MapAlert> {
        let (Some(kind), Some(delivery), Some(mention), Some(filter_match), Ok(filters)) = (
            AlertKind::from_db(&self.kind),
            AlertDelivery::from_db(&self.delivery),
            AlertMention::from_db(&self.mention),
            filters::Match::from_db(&self.filter_match),
            serde_json::from_value(self.filters),
        ) else {
            tracing::warn!(
                "alerts: hiding alert {} ({}): row does not decode",
                self.id,
                self.name
            );
            return None;
        };
        Some(MapAlert {
            id: self.id,
            map_id: self.map_id,
            name: self.name,
            kind,
            delivery,
            map_webhook_id: self.map_webhook_id,
            webhook_name: self.webhook_name,
            discord_channel_id: self.discord_channel_id,
            map_webhook_role_id: self.map_webhook_role_id,
            role_name: self.role_name,
            mention,
            target_solar_system_id: self.target_solar_system_id,
            target_system_name: self.target_system_name,
            origin_solar_system_id: self.origin_solar_system_id,
            origin_system_name: self.origin_system_name,
            max_jumps: self.max_jumps,
            ship_type: self.ship_type.as_deref().and_then(JumpShip::from_db),
            jdc_level: self.jdc_level,
            filters,
            filter_match,
            is_active: self.is_active,
            disabled_reason: self.disabled_reason,
            last_fired_at: self.last_fired_at,
            created_at: self.created_at,
        })
    }
}

async fn rows(pool: &PgPool, map_id: i64, alert_id: Option<i64>) -> Result<Vec<MapAlert>> {
    let rows = sqlx::query_as!(
        AlertRow,
        r#"select a.id, a.map_id, a.name, a.kind, a.delivery,
                  a.map_webhook_id, w.name as "webhook_name?",
                  a.discord_channel_id,
                  a.map_webhook_role_id, r.name as "role_name?",
                  a.mention, a.target_solar_system_id, ss.name as "target_system_name?",
                  a.origin_solar_system_id, os.name as "origin_system_name?",
                  a.max_jumps, a.ship_type, a.jdc_level,
                  a.filters, a.filter_match, a.is_active,
                  a.disabled_reason, a.last_fired_at, a.created_at
           from map_alerts a
           left join solar_systems ss on ss.id = a.target_solar_system_id
           left join solar_systems os on os.id = a.origin_solar_system_id
           left join map_webhooks w on w.id = a.map_webhook_id
           left join map_webhook_roles r on r.id = a.map_webhook_role_id
           where a.map_id = $1 and ($2::bigint is null or a.id = $2)
           order by a.id"#,
        map_id,
        alert_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().filter_map(AlertRow::decode).collect())
}

/// Every alert on the map.
pub async fn list(pool: &PgPool, map_id: i64) -> Result<Vec<MapAlert>> {
    rows(pool, map_id, None).await
}

pub async fn get(pool: &PgPool, map_id: i64, alert_id: i64) -> Result<MapAlert> {
    rows(pool, map_id, Some(alert_id))
        .await?
        .into_iter()
        .next()
        .ok_or(MapError::NotFound)
}

/// What a saved alert must satisfy, wherever it was written: the settings page and the
/// Discord bot share one rule.
pub fn validate(body: &SaveAlert) -> Result<()> {
    let invalid = |message: &str| Err(MapError::Validation(message.into()));
    if body.name.trim().is_empty() {
        return invalid("an alert needs a name");
    }
    if !(0..=30).contains(&body.max_jumps) {
        return invalid("jumps must be between 0 and 30");
    }
    match body.delivery {
        AlertDelivery::Webhook => {
            if body.map_webhook_id.is_none() {
                return invalid("pick a destination");
            }
        }
        AlertDelivery::DiscordChannel => {
            if body.discord_channel_id.is_none() {
                return invalid("pick a channel to post in");
            }
        }
        AlertDelivery::DiscordDm => {}
    }
    if body.mention == AlertMention::Role && body.map_webhook_role_id.is_none() {
        return invalid("pick a role to mention");
    }
    // Proximity and jump range are about a place; a killmail alert is about who.
    if matches!(body.kind, AlertKind::Proximity | AlertKind::JumpRange)
        && body.target_solar_system_id.is_none()
    {
        return invalid("pick a system to watch");
    }
    if body.kind != AlertKind::Proximity && body.origin_solar_system_id.is_some() {
        return invalid("only a proximity alert has a starting point");
    }
    if body.kind == AlertKind::JumpRange {
        if body.ship_type.is_none() {
            return invalid("pick the ship whose range to measure");
        }
        if !(0..=5).contains(&body.jdc_level.unwrap_or(-1)) {
            return invalid("JDC level must be between 0 and 5");
        }
    }
    Ok(())
}

/// A destination or role from another map would be a way to post into a Discord server
/// you were never given, so both are checked to belong here.
pub async fn check_belongs(pool: &PgPool, map_id: i64, body: &SaveAlert) -> Result<()> {
    if let Some(id) = body.map_webhook_id {
        let ok = sqlx::query_scalar!(
            "select exists(select 1 from map_webhooks where id = $1 and map_id = $2)",
            id,
            map_id,
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false);
        if !ok {
            return Err(MapError::Validation(
                "that destination is not on this map".into(),
            ));
        }
    }
    if let Some(id) = body.map_webhook_role_id {
        let ok = sqlx::query_scalar!(
            "select exists(select 1 from map_webhook_roles where id = $1 and map_id = $2)",
            id,
            map_id,
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false);
        if !ok {
            return Err(MapError::Validation("that role is not on this map".into()));
        }
    }
    Ok(())
}

/// A system id that names nothing would only surface as a foreign key error on insert.
pub async fn check_systems_exist(pool: &PgPool, body: &SaveAlert) -> Result<()> {
    for id in [body.target_solar_system_id, body.origin_solar_system_id]
        .into_iter()
        .flatten()
    {
        let ok = sqlx::query_scalar!(
            "select exists(select 1 from solar_systems where id = $1)",
            id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false);
        if !ok {
            return Err(MapError::Validation("that system does not exist".into()));
        }
    }
    Ok(())
}

/// Every check a save has to pass, in the order that gives the clearest message.
pub async fn check(pool: &PgPool, map_id: i64, body: &SaveAlert) -> Result<()> {
    validate(body)?;
    check_belongs(pool, map_id, body).await?;
    check_systems_exist(pool, body).await
}

/// The settings page's half of a bot channel alert. The bot posts wherever it has been
/// invited, so a channel, and who it may ping there, are only ever taken from Discord: the
/// `/wh alert-channel` command runs in the channel, signed by Discord, with the invoking member's
/// permissions. Over HTTP nobody can create one or point an alert at a channel, and editing
/// one keeps the channel and the ping it was given (dropping the ping is fine). Everything
/// else about it stays editable. `alert_id` is `None` for a new alert.
pub async fn keep_bot_destination(
    pool: &PgPool,
    map_id: i64,
    alert_id: Option<i64>,
    body: &mut SaveAlert,
) -> Result<()> {
    body.discord_guild_id = None;
    body.discord_channel_id = None;
    if body.delivery != AlertDelivery::DiscordChannel {
        return Ok(());
    }
    let from_discord = || {
        Err(MapError::Validation(
            "a bot channel alert is set up with /wh alert-channel in the Discord channel it posts to".into(),
        ))
    };
    let Some(alert_id) = alert_id else {
        return from_discord();
    };
    let stored = sqlx::query!(
        "select delivery, discord_guild_id, discord_channel_id, mention, map_webhook_role_id
         from map_alerts where id = $1 and map_id = $2",
        alert_id,
        map_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(MapError::NotFound)?;
    if AlertDelivery::from_db(&stored.delivery) != Some(AlertDelivery::DiscordChannel) {
        return from_discord();
    }
    let same_ping = AlertMention::from_db(&stored.mention) == Some(body.mention)
        && stored.map_webhook_role_id == body.map_webhook_role_id;
    if !same_ping && body.mention != AlertMention::None {
        return Err(MapError::Validation(
            "who a bot channel alert pings is set with /wh alert-channel in Discord".into(),
        ));
    }
    if body.mention == AlertMention::None {
        body.map_webhook_role_id = None;
    }
    body.discord_guild_id = stored.discord_guild_id;
    body.discord_channel_id = stored.discord_channel_id;
    Ok(())
}

/// Inserts the alert and logs who set it up. The caller runs [`check`] first.
pub async fn create(
    pool: &PgPool,
    map_id: i64,
    user_id: i64,
    body: &SaveAlert,
) -> Result<MapAlert> {
    let id = sqlx::query_scalar!(
        "insert into map_alerts
             (map_id, created_by_user_id, name, kind, delivery, map_webhook_id,
              discord_guild_id, discord_channel_id, map_webhook_role_id, mention,
              target_solar_system_id, origin_solar_system_id, max_jumps, ship_type, jdc_level,
              filters, filter_match)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
         returning id",
        map_id,
        user_id,
        body.name.trim(),
        body.kind.as_str(),
        body.delivery.as_str(),
        body.map_webhook_id,
        body.discord_guild_id,
        body.discord_channel_id,
        body.map_webhook_role_id,
        body.mention.as_str(),
        body.target_solar_system_id,
        body.origin_solar_system_id,
        body.max_jumps,
        body.ship_type.map(|v| v.as_str()),
        body.jdc_level,
        serde_json::to_value(&body.filters).unwrap_or_else(|_| serde_json::json!([])),
        body.filter_match.as_str(),
    )
    .fetch_one(pool)
    .await?;
    crate::alerts::log(pool, Some(id), map_id, Some(user_id), "created", None).await;
    get(pool, map_id, id).await
}

/// Replaces the alert's settings. The caller runs [`check`] first.
pub async fn update(
    pool: &PgPool,
    map_id: i64,
    alert_id: i64,
    user_id: i64,
    body: &SaveAlert,
) -> Result<MapAlert> {
    let updated = sqlx::query!(
        "update map_alerts set
             name = $3, kind = $4, delivery = $5, map_webhook_id = $6,
             discord_guild_id = $7, discord_channel_id = $8, map_webhook_role_id = $9,
             mention = $10, target_solar_system_id = $11, origin_solar_system_id = $12,
             max_jumps = $13, ship_type = $14, jdc_level = $15,
             filters = $16, filter_match = $17, updated_at = now()
         where id = $1 and map_id = $2",
        alert_id,
        map_id,
        body.name.trim(),
        body.kind.as_str(),
        body.delivery.as_str(),
        body.map_webhook_id,
        body.discord_guild_id,
        body.discord_channel_id,
        body.map_webhook_role_id,
        body.mention.as_str(),
        body.target_solar_system_id,
        body.origin_solar_system_id,
        body.max_jumps,
        body.ship_type.map(|v| v.as_str()),
        body.jdc_level,
        serde_json::to_value(&body.filters).unwrap_or_else(|_| serde_json::json!([])),
        body.filter_match.as_str(),
    )
    .execute(pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(MapError::NotFound);
    }
    crate::alerts::log(pool, Some(alert_id), map_id, Some(user_id), "updated", None).await;
    get(pool, map_id, alert_id).await
}

/// Turns the alert on or off by hand. Turning it back on clears the reason it stopped.
/// `via` notes where it was done from, when that was not the settings page.
pub async fn set_active(
    pool: &PgPool,
    map_id: i64,
    alert_id: i64,
    user_id: i64,
    is_active: bool,
    via: Option<&str>,
) -> Result<MapAlert> {
    let updated = sqlx::query!(
        "update map_alerts set
             is_active = $3,
             disabled_at = case when $3 then null else now() end,
             disabled_reason = case when $3 then null else 'manual' end,
             updated_at = now()
         where id = $1 and map_id = $2",
        alert_id,
        map_id,
        is_active,
    )
    .execute(pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(MapError::NotFound);
    }
    crate::alerts::log(
        pool,
        Some(alert_id),
        map_id,
        Some(user_id),
        if is_active { "enabled" } else { "disabled" },
        via,
    )
    .await;
    get(pool, map_id, alert_id).await
}

pub async fn delete(pool: &PgPool, map_id: i64, alert_id: i64, user_id: i64) -> Result<()> {
    let name = sqlx::query_scalar!(
        "delete from map_alerts where id = $1 and map_id = $2 returning name",
        alert_id,
        map_id,
    )
    .fetch_optional(pool)
    .await?;
    let Some(name) = name else {
        return Err(MapError::NotFound);
    };
    crate::alerts::log(pool, None, map_id, Some(user_id), "deleted", Some(&name)).await;
    Ok(())
}

/// Register a Discord role the map's alerts can mention, or rename it if the map has it
/// already. Returns the role's id on this map.
pub async fn register_role(
    pool: &PgPool,
    map_id: i64,
    name: &str,
    discord_role_id: &str,
) -> Result<i64> {
    Ok(sqlx::query_scalar!(
        "insert into map_webhook_roles (map_id, name, discord_role_id) values ($1, $2, $3)
         on conflict (map_id, discord_role_id) do update set name = excluded.name
         returning id",
        map_id,
        name,
        discord_role_id,
    )
    .fetch_one(pool)
    .await?)
}

/// One line of an alert's history.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapAlertEvent {
    pub id: i64,
    #[ts(optional)]
    pub map_alert_id: Option<i64>,
    #[ts(optional)]
    pub alert_name: Option<String>,
    #[ts(optional)]
    pub actor: Option<String>,
    pub kind: String,
    #[ts(optional)]
    pub detail: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A registered destination, named once and pointed at by any number of alerts.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapWebhook {
    pub id: i64,
    pub name: String,
    /// Enough of the URL to tell two destinations apart, never enough to use one: the URL is
    /// a bearer token for somebody's channel, and every manager reads this list.
    pub summary: String,
    /// How many alerts would stop working if this were deleted.
    pub alert_count: i64,
}

/// A registered role, so alerts ping "Scouts" rather than 1189734502938472.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapWebhookRole {
    pub id: i64,
    pub name: String,
    pub discord_role_id: String,
}

fn webhook_summary(url: &str) -> String {
    let rest = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = rest.split('/').next().unwrap_or(rest);
    // The id is stable and public-ish; the token after it is the secret.
    let id = rest
        .split('/')
        .nth(3)
        .filter(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    match id {
        Some(id) => format!("{host}/…/{id}"),
        None => host.to_string(),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SaveWebhook {
    pub name: String,
    /// Write-only. Absent on an update keeps the stored one.
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SaveRole {
    pub name: String,
    pub discord_role_id: String,
}

/// The map's destinations, each with how many alerts point at it.
pub async fn list_webhooks(pool: &PgPool, map_id: i64) -> Result<Vec<MapWebhook>> {
    let rows = sqlx::query!(
        r#"select w.id, w.name, w.url,
                  (select count(*) from map_alerts a where a.map_webhook_id = w.id) as "alert_count!"
           from map_webhooks w where w.map_id = $1 order by w.name"#,
        map_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| MapWebhook {
            id: row.id,
            name: row.name,
            summary: webhook_summary(&row.url),
            alert_count: row.alert_count,
        })
        .collect())
}

/// Register a destination, or point an existing name at a new URL.
pub async fn create_webhook(
    pool: &PgPool,
    map_id: i64,
    user_id: i64,
    body: &SaveWebhook,
) -> Result<MapWebhook> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(MapError::Validation("give the destination a name".into()));
    }
    let Some(url) = body.url.as_deref().map(str::trim).filter(|u| !u.is_empty()) else {
        return Err(MapError::Validation(
            "paste the channel's webhook URL".into(),
        ));
    };
    if !url.starts_with("https://discord.com/api/webhooks/") {
        return Err(MapError::Validation(
            "that is not a Discord webhook URL".into(),
        ));
    }
    let id = sqlx::query_scalar!(
        "insert into map_webhooks (map_id, name, url) values ($1, $2, $3)
         on conflict (map_id, name) do update set url = excluded.url, updated_at = now()
         returning id",
        map_id,
        name,
        url,
    )
    .fetch_one(pool)
    .await?;
    crate::alerts::log(pool, None, map_id, Some(user_id), "destination", Some(name)).await;
    Ok(MapWebhook {
        id,
        name: name.to_string(),
        summary: webhook_summary(url),
        alert_count: 0,
    })
}

/// Delete a destination. Alerts pointing at it go with it, rather than staying enabled
/// with nowhere to post.
pub async fn delete_webhook(
    pool: &PgPool,
    map_id: i64,
    webhook_id: i64,
    user_id: i64,
) -> Result<()> {
    let name = sqlx::query_scalar!(
        "delete from map_webhooks where id = $1 and map_id = $2 returning name",
        webhook_id,
        map_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(MapError::NotFound)?;
    crate::alerts::log(
        pool,
        None,
        map_id,
        Some(user_id),
        "destination_deleted",
        Some(&name),
    )
    .await;
    Ok(())
}

/// The map's named Discord roles.
pub async fn list_roles(pool: &PgPool, map_id: i64) -> Result<Vec<MapWebhookRole>> {
    let rows = sqlx::query!(
        "select id, name, discord_role_id from map_webhook_roles
         where map_id = $1 order by name",
        map_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| MapWebhookRole {
            id: row.id,
            name: row.name,
            discord_role_id: row.discord_role_id,
        })
        .collect())
}

/// Name a role so alerts can ping it.
pub async fn create_role(pool: &PgPool, map_id: i64, body: &SaveRole) -> Result<MapWebhookRole> {
    let name = body.name.trim();
    let role_id = body.discord_role_id.trim();
    if name.is_empty() {
        return Err(MapError::Validation("give the role a name".into()));
    }
    // Discord ids are snowflakes: decimal, and long. Anything else is a copy-paste slip.
    if role_id.len() < 5 || !crate::discord::is_snowflake(role_id) {
        return Err(MapError::Validation(
            "a Discord role id is a long number: right-click the role with developer mode on"
                .into(),
        ));
    }
    let id = register_role(pool, map_id, name, role_id).await?;
    Ok(MapWebhookRole {
        id,
        name: name.to_string(),
        discord_role_id: role_id.to_string(),
    })
}

pub async fn delete_role(pool: &PgPool, map_id: i64, role_id: i64) -> Result<()> {
    let deleted = sqlx::query!(
        "delete from map_webhook_roles where id = $1 and map_id = $2",
        role_id,
        map_id,
    )
    .execute(pool)
    .await?;
    if deleted.rows_affected() == 0 {
        return Err(MapError::NotFound);
    }
    Ok(())
}

/// The newest lines of the map's alert audit trail, 50 unless asked for more (at most 200).
pub async fn list_events(
    pool: &PgPool,
    map_id: i64,
    limit: Option<i64>,
) -> Result<Vec<MapAlertEvent>> {
    let limit = limit.unwrap_or(50).clamp(1, 200);
    let rows = sqlx::query!(
        r#"select e.id, e.map_alert_id, a.name as "alert_name?", e.kind, e.detail, e.created_at,
                  (select c.name from characters c
                    where c.user_id = e.actor_user_id
                    order by c.id limit 1) as "actor?"
           from map_alert_events e
           left join map_alerts a on a.id = e.map_alert_id
           where e.map_id = $1
           order by e.id desc
           limit $2"#,
        map_id,
        limit,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| MapAlertEvent {
            id: row.id,
            map_alert_id: row.map_alert_id,
            alert_name: row.alert_name,
            actor: row.actor,
            kind: row.kind,
            detail: row.detail,
            created_at: row.created_at,
        })
        .collect())
}

#[cfg(test)]
mod webhook_tests {
    use super::*;

    #[test]
    fn a_webhook_is_summarised_without_its_token() {
        let url = "https://discord.com/api/webhooks/123456/verysecrettoken";
        let shown = webhook_summary(url);
        assert!(shown.contains("discord.com"));
        assert!(shown.contains("123456"));
        assert!(!shown.contains("verysecrettoken"));
    }
}
