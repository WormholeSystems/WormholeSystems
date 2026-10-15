//! The Discord side of the integration: linking an account, and answering slash commands.
//!
//! No gateway process. Discord delivers interactions over HTTPS to an endpoint that proves
//! it holds the application's key, so the bot is a route in this server rather than a second
//! long-lived thing to deploy, reconnect and supervise.

pub mod commands;
pub mod interactions;
pub mod link;

use serde::Deserialize;

/// Discord's API root. Not overridable: the tests here are about signature verification and
/// command shapes, not transport.
pub const API: &str = "https://discord.com/api/v10";

/// Whether `id` is a Discord id: a snowflake, which is a decimal number of at most 20
/// digits. Checked before an id goes into a URL path, where anything else could point a
/// request somewhere it was never meant to go.
pub fn is_snowflake(id: &str) -> bool {
    (1..=20).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_digit())
}

/// The bits of a Discord user WormholeSystems stores.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    /// The display name, when they have set one.
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

/// A linked account, as the frontend shows it.
#[derive(Debug, Clone, serde::Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct DiscordAccount {
    pub discord_user_id: String,
    pub username: String,
    #[ts(optional)]
    pub display_name: Option<String>,
    #[ts(optional)]
    pub avatar: Option<String>,
}

/// The linked account for a user, if any.
pub async fn account_for(pool: &sqlx::PgPool, user_id: i64) -> Option<DiscordAccount> {
    sqlx::query!(
        "select discord_user_id, username, display_name, avatar
         from discord_accounts where user_id = $1",
        user_id,
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .map(|row| DiscordAccount {
        discord_user_id: row.discord_user_id,
        username: row.username,
        display_name: row.display_name,
        avatar: row.avatar,
    })
}

/// The user behind a Discord id, if they have linked.
pub async fn user_for(pool: &sqlx::PgPool, discord_user_id: &str) -> Option<i64> {
    sqlx::query_scalar!(
        "select user_id from discord_accounts where discord_user_id = $1",
        discord_user_id,
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}
