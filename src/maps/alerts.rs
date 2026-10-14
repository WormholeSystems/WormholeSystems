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
    #[serde(default)]
    #[ts(optional)]
    pub discord_guild_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
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
