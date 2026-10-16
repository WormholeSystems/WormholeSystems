//! A person's display preferences, kept on their account so they follow them to every
//! device and every map. Per-map choices live in `map_user_settings` instead.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UserPreferences {
    /// The game's icon on a wormhole effect badge; off shows the effect's letter instead.
    pub effect_icons: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        UserPreferences { effect_icons: true }
    }
}

/// A partial change: a field left out stays as it is.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UpdateUserPreferences {
    #[serde(default)]
    #[ts(optional)]
    pub effect_icons: Option<bool>,
}

/// The user's preferences, the defaults until they change one.
pub async fn get(pool: &PgPool, user_id: i64) -> Result<UserPreferences, sqlx::Error> {
    let row = sqlx::query_as!(
        UserPreferences,
        "select effect_icons from user_preferences where user_id = $1",
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.unwrap_or_default())
}

/// Apply a change and return the result.
pub async fn update(
    pool: &PgPool,
    user_id: i64,
    change: UpdateUserPreferences,
) -> Result<UserPreferences, sqlx::Error> {
    let defaults = UserPreferences::default();
    sqlx::query_as!(
        UserPreferences,
        "insert into user_preferences (user_id, effect_icons)
         values ($1, coalesce($2::bool, $3))
         on conflict (user_id) do update set
             effect_icons = coalesce($2::bool, user_preferences.effect_icons),
             updated_at = now()
         returning effect_icons",
        user_id,
        change.effect_icons,
        defaults.effect_icons,
    )
    .fetch_one(pool)
    .await
}
