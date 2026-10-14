//! The per-user settings kept against a map: what each viewer has chosen to see, where
//! they have put their panels, and the toggles that are theirs alone rather than the
//! map's.
//!
//! The defaults live in two places that have to agree: the columns' own defaults in the
//! migrations, which a new row gets, and [`MapUserSettings::default`], which a user with
//! no row yet is shown. A test holds them together.

pub mod panels;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::error::{MapError, Result};
use super::{BackgroundMode, KillmailScope, MapLayout, MassStatus, RoutePreference, TimeStatus};
use panels::{PanelLayouts, validate_layouts};

/// A user's per-map preferences.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapUserSettings {
    pub tracking_allowed: bool,
    pub show_threat_level: bool,
    pub compact_signature_list: bool,
    pub show_statics_first: bool,
    pub route_preference: RoutePreference,
    /// 0-100, weight of the security preference (legacy `exp(0.15 * penalty)`).
    pub security_penalty: i32,
    /// Worst wormhole lifetime still routed through.
    pub route_allow_time_status: TimeStatus,
    /// Worst wormhole mass still routed through.
    pub route_allow_mass_status: MassStatus,
    pub route_use_evescout: bool,
    /// Whether the chain's own holes are route edges. Off routes over stargates alone.
    pub route_use_wormholes: bool,
    /// Ask which signature was jumped, rather than mapping the hole unlinked.
    pub prompt_for_signature: bool,
    /// Open that prompt with the likeliest signature already chosen.
    pub preselect_signature: bool,
    /// Prefill the jump dialog's alias from the chain's naming scheme.
    pub suggest_alias: bool,
    /// Put the new connection's bookmark on the clipboard once the jump is mapped.
    pub copy_bookmark: bool,
    /// Select the system this user's pilot is in as they fly, so the cards follow them.
    pub follow_character: bool,
    /// The pilots whose jumps build this map. Empty = the character the session acts as.
    pub tracked_character_ids: Vec<i64>,
    /// Which half of the chain the killmails card shows.
    pub killmail_filter: KillmailScope,
    pub is_archived: bool,
    /// Whether this user has been through the map's introduction.
    pub introduction_confirmed: bool,
    /// Panels this user hides on this map. Empty = the built-in set. A hidden panel keeps
    /// its saved position, so unhiding puts it back where it was.
    pub hidden_panels: Vec<String>,
    /// This viewer's placement choice, when the map hands it to them. `None` follows the
    /// map's own mode.
    #[ts(optional)]
    pub layout_override: Option<MapLayout>,
    /// Whether this user keeps the map in the top bar for quick access.
    pub is_pinned: bool,
    /// Per-breakpoint tile positions. `None` = the built-in arrangement.
    #[ts(optional)]
    pub layout_breakpoints: Option<PanelLayouts>,
    /// Where to fetch the picture behind this user's map from, or `None` for the plain
    /// grid. Changes with every upload, so it is safe to cache hard.
    pub background_image_url: Option<String>,
    pub background_image_mode: BackgroundMode,
}

/// Partial update of [`MapUserSettings`]; absent fields stay unchanged.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UpdateMapUserSettings {
    #[serde(default)]
    #[ts(optional)]
    pub is_pinned: Option<bool>,
    /// Absent leaves it; `null` goes back to following the map.
    #[serde(default, deserialize_with = "crate::maps::double_option")]
    #[ts(optional)]
    pub layout_override: Option<Option<MapLayout>>,
    #[serde(default)]
    #[ts(optional)]
    pub tracking_allowed: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub show_threat_level: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub compact_signature_list: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub show_statics_first: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub route_preference: Option<RoutePreference>,
    #[serde(default)]
    #[ts(optional)]
    pub security_penalty: Option<i32>,
    #[serde(default)]
    #[ts(optional)]
    pub route_allow_time_status: Option<TimeStatus>,
    #[serde(default)]
    #[ts(optional)]
    pub route_allow_mass_status: Option<MassStatus>,
    #[serde(default)]
    #[ts(optional)]
    pub route_use_evescout: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub route_use_wormholes: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub prompt_for_signature: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub preselect_signature: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub suggest_alias: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub copy_bookmark: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub follow_character: Option<bool>,
    /// Replaces the set; every id must be one of the caller's characters.
    #[serde(default)]
    #[ts(optional)]
    pub tracked_character_ids: Option<Vec<i64>>,
    #[serde(default)]
    #[ts(optional)]
    pub killmail_filter: Option<KillmailScope>,
    #[serde(default)]
    #[ts(optional)]
    pub is_archived: Option<bool>,
    /// Stamped server-side, so "when" is the server's clock rather than the browser's.
    #[serde(default)]
    #[ts(optional)]
    pub introduction_confirmed: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub hidden_panels: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub layout_breakpoints: Option<PanelLayouts>,
    /// The image itself goes through `/api/maps/{id}/background-image`; this is only how
    /// it is laid out.
    #[serde(default)]
    #[ts(optional)]
    pub background_image_mode: Option<BackgroundMode>,
}

/// The URL the stored file is fetched from. The file name is the cache key: a replacement
/// is a new file, so a browser that cached the old one asks again.
pub fn background_image_url(map_id: i64, path: Option<&str>) -> Option<String> {
    let file = path?.rsplit('/').next()?;
    Some(format!("/api/maps/{map_id}/background-image?v={file}"))
}

/// What a user who never changed anything has. Matches the column defaults.
impl Default for MapUserSettings {
    fn default() -> Self {
        MapUserSettings {
            layout_override: None,
            is_pinned: false,
            tracking_allowed: false,
            show_threat_level: true,
            compact_signature_list: false,
            show_statics_first: false,
            route_preference: RoutePreference::Shorter,
            security_penalty: 50,
            route_allow_time_status: TimeStatus::Critical,
            route_allow_mass_status: MassStatus::Reduced,
            route_use_evescout: false,
            route_use_wormholes: true,
            prompt_for_signature: true,
            preselect_signature: false,
            suggest_alias: true,
            copy_bookmark: false,
            follow_character: false,
            tracked_character_ids: Vec::new(),
            killmail_filter: KillmailScope::All,
            is_archived: false,
            introduction_confirmed: false,
            hidden_panels: Vec::new(),
            layout_breakpoints: None,
            background_image_url: None,
            background_image_mode: BackgroundMode::Grid,
        }
    }
}

/// A user's settings on a map, or [the defaults](MapUserSettings::default) when no row
/// exists yet.
pub async fn load(db: &PgPool, map_id: i64, user_id: i64) -> Result<MapUserSettings> {
    let row = sqlx::query!(
        r#"select tracking_allowed, show_threat_level, compact_signature_list,
                  show_statics_first,
                  route_preference,
                  security_penalty,
                  route_allow_time_status,
                  route_allow_mass_status,
                  route_use_evescout,
                  route_use_wormholes,
                  prompt_for_signature, preselect_signature, suggest_alias, copy_bookmark,
                  follow_character, tracked_character_ids,
                  killmail_filter,
                  is_archived,
                  (introduction_confirmed_at is not null) as "introduction_confirmed!",
                  hidden_panels, layout_breakpoints,
                  layout_override, is_pinned,
                  background_image_path,
                  background_image_mode
           from map_user_settings where map_id = $1 and user_id = $2"#,
        map_id,
        user_id,
    )
    .fetch_optional(db)
    .await?;
    Ok(match row {
        Some(r) => MapUserSettings {
            tracking_allowed: r.tracking_allowed,
            show_threat_level: r.show_threat_level,
            compact_signature_list: r.compact_signature_list,
            show_statics_first: r.show_statics_first,
            route_preference: r.route_preference,
            security_penalty: r.security_penalty,
            route_allow_time_status: r.route_allow_time_status,
            route_allow_mass_status: r.route_allow_mass_status,
            route_use_evescout: r.route_use_evescout,
            route_use_wormholes: r.route_use_wormholes,
            prompt_for_signature: r.prompt_for_signature,
            preselect_signature: r.preselect_signature,
            suggest_alias: r.suggest_alias,
            copy_bookmark: r.copy_bookmark,
            follow_character: r.follow_character,
            tracked_character_ids: r.tracked_character_ids,
            killmail_filter: r.killmail_filter,
            is_archived: r.is_archived,
            introduction_confirmed: r.introduction_confirmed,
            hidden_panels: r.hidden_panels,
            layout_override: r.layout_override,
            is_pinned: r.is_pinned,
            layout_breakpoints: r
                .layout_breakpoints
                .map(serde_json::from_value)
                .transpose()
                .unwrap_or(None),
            background_image_url: background_image_url(map_id, r.background_image_path.as_deref()),
            background_image_mode: r.background_image_mode,
        },
        None => MapUserSettings::default(),
    })
}

/// Apply a partial update to the user's settings on a map, and hand back the result. The
/// caller has checked the user can see the map.
///
/// A first change inserts the bare row, so every column it does not name takes the
/// column's default, and then sets only what was sent.
pub async fn update(
    pool: &PgPool,
    map_id: i64,
    user_id: i64,
    body: &UpdateMapUserSettings,
) -> Result<MapUserSettings> {
    // The tolerances and the preference are enums, so a value outside them never gets this
    // far: serde rejects the body. Only the number still needs saying out loud.
    if let Some(p) = body.security_penalty
        && !(0..=100).contains(&p)
    {
        return Err(MapError::Validation(
            "security penalty must be 0-100".into(),
        ));
    }
    if let Some(ids) = &body.tracked_character_ids {
        let owned = sqlx::query_scalar!(
            "select count(*) from characters where user_id = $1 and id = any($2)",
            user_id,
            ids,
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);
        if owned as usize != ids.len() {
            return Err(MapError::Validation(
                "only your own characters can map for you".into(),
            ));
        }
    }
    // Reject an arrangement that could not render, rather than breaking the next load.
    let layout_json =
        match &body.layout_breakpoints {
            Some(layouts) => {
                validate_layouts(layouts)?;
                Some(serde_json::to_value(layouts).map_err(|e| {
                    MapError::Validation(format!("could not store the layout: {e}"))
                })?)
            }
            None => None,
        };

    let mut tx = pool.begin().await?;
    insert_defaults(&mut *tx, map_id, user_id).await?;
    sqlx::query!(
        r#"update map_user_settings set
             tracking_allowed = coalesce($3, tracking_allowed),
             show_threat_level = coalesce($4, show_threat_level),
             compact_signature_list = coalesce($5, compact_signature_list),
             show_statics_first = coalesce($6, show_statics_first),
             route_preference = coalesce($7, route_preference),
             security_penalty = coalesce($8, security_penalty),
             route_allow_time_status = coalesce($9, route_allow_time_status),
             route_allow_mass_status = coalesce($10, route_allow_mass_status),
             route_use_evescout = coalesce($11, route_use_evescout),
             prompt_for_signature = coalesce($12, prompt_for_signature),
             suggest_alias = coalesce($13, suggest_alias),
             copy_bookmark = coalesce($14, copy_bookmark),
             killmail_filter = coalesce($15, killmail_filter),
             is_archived = coalesce($16, is_archived),
             -- Absent leaves it; true stamps now; false clears it and shows it again.
             introduction_confirmed_at = case
                 when $17::boolean is null then introduction_confirmed_at
                 when $17 then now()
             end,
             hidden_panels = coalesce($18, hidden_panels),
             layout_breakpoints = coalesce($19, layout_breakpoints),
             -- Absent leaves it; null is a real value here (follow the map again), so it
             -- cannot go through coalesce.
             layout_override = case when $21 then $20 else layout_override end,
             is_pinned = coalesce($22, is_pinned),
             follow_character = coalesce($23, follow_character),
             tracked_character_ids = coalesce($24, tracked_character_ids),
             background_image_mode = coalesce($25, background_image_mode),
             preselect_signature = coalesce($26, preselect_signature),
             route_use_wormholes = coalesce($27, route_use_wormholes),
             updated_at = now()
           where map_id = $1 and user_id = $2"#,
        map_id,
        user_id,
        body.tracking_allowed,
        body.show_threat_level,
        body.compact_signature_list,
        body.show_statics_first,
        body.route_preference as Option<RoutePreference>,
        body.security_penalty,
        body.route_allow_time_status as Option<TimeStatus>,
        body.route_allow_mass_status as Option<MassStatus>,
        body.route_use_evescout,
        body.prompt_for_signature,
        body.suggest_alias,
        body.copy_bookmark,
        body.killmail_filter as Option<KillmailScope>,
        body.is_archived,
        body.introduction_confirmed,
        body.hidden_panels.as_deref(),
        layout_json.as_ref(),
        body.layout_override.flatten() as Option<MapLayout>,
        body.layout_override.is_some(),
        body.is_pinned,
        body.follow_character,
        body.tracked_character_ids.as_deref(),
        body.background_image_mode as Option<BackgroundMode>,
        body.preselect_signature,
        body.route_use_wormholes,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    // Whether this user shares their position decides whether they appear on everyone
    // else's pilot list, and the poller only announces maps that are already shared: with
    // nothing published here, switching it off never reaches the other viewers at all.
    if body.tracking_allowed.is_some() {
        crate::realtime::hub().publish(crate::realtime::MapEvent::CharactersChanged { map_id });
    }

    load(pool, map_id, user_id).await
}

/// The bare row: the key and nothing else, so every setting is the column's default.
/// Nothing when the row is there already.
pub async fn insert_defaults<'e, E>(executor: E, map_id: i64, user_id: i64) -> Result<()>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query!(
        "insert into map_user_settings (map_id, user_id) values ($1, $2)
         on conflict (map_id, user_id) do nothing",
        map_id,
        user_id,
    )
    .execute(executor)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::background_image_url;

    #[test]
    fn the_url_carries_the_file_name_so_a_replacement_is_a_new_url() {
        assert_eq!(background_image_url(4, None), None);
        assert_eq!(
            background_image_url(4, Some("map-backgrounds/4/9-1700000000000.png")).as_deref(),
            Some("/api/maps/4/background-image?v=9-1700000000000.png")
        );
    }
}
