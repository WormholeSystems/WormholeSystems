//! A user's characters: who is signed in, what each is doing, what they have consented
//! to, taking one off the account, and the ESI calls that act on one character alone
//! (autopilot waypoints). Nothing here is about a map.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::allowlist::AffiliationAllowlist;
use crate::db::PgTokenStore;
use crate::esi::{EsiClient, EsiError, Scope, Sso};
use crate::maps::Actor;

/// Why a character operation was refused.
#[derive(Debug, thiserror::Error)]
pub enum CharacterError {
    /// Something the user can fix: a character that is not theirs, the last one, an
    /// unknown destination.
    #[error("{0}")]
    Invalid(String),
    /// The character has to re-consent before this can work.
    #[error("that character has not granted the waypoint scope")]
    MissingScope,
    /// ESI refused or failed. The detail is for the log, not the caller.
    #[error("ESI failed: {0}")]
    Upstream(EsiError),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

type Result<T> = std::result::Result<T, CharacterError>;

fn invalid(message: &str) -> CharacterError {
    CharacterError::Invalid(message.to_string())
}

/// The signed-in character, for the UI's auth state.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CharacterSummary {
    pub character_id: i64,
    pub name: String,
    /// Whether this character may create a map here. An instance can limit that to named
    /// affiliations, and a button that only produces a 403 is worse than no button.
    pub can_create_map: bool,
}

/// Live status of the active character, for the navbar readout.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CharacterStatus {
    pub online: bool,
    pub solar_system: Option<String>,
    pub ship_type_id: Option<i64>,
    pub ship_name: Option<String>,
    pub ship_type: Option<String>,
}

/// One of the user's characters, for the switcher.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CharacterRef {
    pub character_id: i64,
    pub name: String,
    pub is_active: bool,
    /// The one new sessions start as, chosen on the characters page.
    pub is_preferred: bool,
    pub online: bool,
    /// Where the character is right now, when online and tracked. Drives the paste
    /// system-mismatch warning.
    pub solar_system_id: Option<i64>,
    /// EVE refused this character's token (revoked, or rotated away), so it is not tracked
    /// and cannot act until it logs in again.
    pub needs_login: bool,
}

/// One ESI permission and whether the acting character has consented to it.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ScopeStatus {
    pub scope: String,
    pub granted: bool,
}

/// The acting character as the UI shows it, or `None` when the character is gone.
pub async fn summary(
    pool: &PgPool,
    actor: Actor,
    map_creators: &AffiliationAllowlist,
) -> Result<Option<CharacterSummary>> {
    let name = sqlx::query_scalar!(
        "select name from characters where id = $1",
        actor.character_id
    )
    .fetch_optional(pool)
    .await?;
    let Some(name) = name else {
        return Ok(None);
    };
    let can_create_map = crate::maps::map::may_create_map(pool, actor, map_creators)
        .await
        .map_err(|err| match err {
            crate::maps::MapError::Db(err) => CharacterError::Db(err),
            other => CharacterError::Invalid(other.to_string()),
        })?;
    Ok(Some(CharacterSummary {
        character_id: actor.character_id,
        name,
        can_create_map,
    }))
}

/// The character's live status, or `None` when the poller has not seen it yet.
pub async fn status(pool: &PgPool, character_id: i64) -> Result<Option<CharacterStatus>> {
    let row = sqlx::query!(
        r#"select s.online,
                  s.ship_type_id,
                  ss.name as "solar_system?",
                  s.ship_name,
                  t.name as "ship_type?"
           from character_status s
           left join solar_systems ss on ss.id = s.solar_system_id
           left join types t on t.id = s.ship_type_id
           where s.character_id = $1"#,
        character_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| CharacterStatus {
        online: r.online,
        solar_system: r.solar_system,
        ship_type_id: r.ship_type_id,
        ship_name: r.ship_name,
        ship_type: r.ship_type,
    }))
}

/// Every character on the actor's account, marking the active and preferred ones.
pub async fn list(pool: &PgPool, actor: Actor) -> Result<Vec<CharacterRef>> {
    let rows = sqlx::query!(
        r#"select c.id, c.name, c.is_preferred, coalesce(s.online, false) as "online!",
                  case when s.online then s.solar_system_id end as "solar_system_id?",
                  exists (select 1 from esi_tokens t
                          where t.character_id = c.id and t.rejected_at is not null)
                      as "needs_login!"
           from characters c
           left join character_status s on s.character_id = c.id
           where c.user_id = $1 order by c.name"#,
        actor.user_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| CharacterRef {
            character_id: r.id,
            name: r.name,
            is_active: r.id == actor.character_id,
            is_preferred: r.is_preferred,
            online: r.online,
            solar_system_id: r.solar_system_id,
            needs_login: r.needs_login,
        })
        .collect())
}

/// The ESI scopes a character has consented to, across every token we hold for them.
pub async fn granted_scopes(pool: &PgPool, character_id: i64) -> Vec<Scope> {
    let names = sqlx::query_scalar!(
        "select distinct s.name
         from esi_tokens t
         join esi_token_scopes ts on ts.token_id = t.id
         join esi_scopes s on s.id = ts.scope_id
         where t.character_id = $1",
        character_id,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    names.iter().filter_map(|n| Scope::parse(n)).collect()
}

/// Every ESI permission the app can use, and whether this character has granted it.
/// Always the full list, so the UI can show what is missing.
pub async fn scopes(pool: &PgPool, character_id: i64) -> Vec<ScopeStatus> {
    let granted = granted_scopes(pool, character_id).await;
    Scope::ALL
        .iter()
        .map(|scope| ScopeStatus {
            scope: scope.as_str().to_string(),
            granted: granted.contains(scope),
        })
        .collect()
}

async fn owns(pool: &PgPool, user_id: i64, character_id: i64) -> Result<bool> {
    Ok(sqlx::query_scalar!(
        "select exists(select 1 from characters where id = $1 and user_id = $2)",
        character_id,
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false))
}

/// Take a character off the actor's account. Refuses the last one. When it is the one
/// this session acts as, the session moves to another first, since sessions go with their
/// character.
pub async fn remove(
    pool: &PgPool,
    session_id: &str,
    actor: Actor,
    character_id: i64,
) -> Result<()> {
    let count = sqlx::query_scalar!(
        "select count(*) from characters where user_id = $1",
        actor.user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);
    if count <= 1 {
        return Err(invalid("can't remove your only character"));
    }
    if !owns(pool, actor.user_id, character_id).await? {
        return Err(invalid("that character isn't yours"));
    }

    // Switch away before deleting the active one (sessions FK-cascade on the character).
    if character_id == actor.character_id
        && let Some(other) = sqlx::query_scalar!(
            "select id from characters where user_id = $1 and id <> $2
             order by is_preferred desc, id limit 1",
            actor.user_id,
            character_id,
        )
        .fetch_optional(pool)
        .await?
    {
        crate::session::set_active_character(pool, session_id, other).await?;
    }

    sqlx::query!(
        "delete from characters where id = $1 and user_id = $2",
        character_id,
        actor.user_id,
    )
    .execute(pool)
    .await?;

    // Removing the preferred character leaves the account without one, so hand the flag on.
    crate::session::ensure_preferred_character(pool, actor.user_id).await?;
    Ok(())
}

/// Where a waypoint goes, and how it joins the route already set.
#[derive(Debug, Clone, Copy)]
pub struct Waypoint {
    pub destination_id: i64,
    pub add_to_beginning: bool,
    pub clear_other_waypoints: bool,
}

/// The two ESI handles a waypoint needs: SSO for the character's token, ESI for the call.
pub struct Esi<'a> {
    pub sso: &'a Sso,
    pub esi: &'a EsiClient,
}

/// Set a waypoint for one of the user's characters.
pub async fn set_waypoint(
    pool: &PgPool,
    esi: Esi<'_>,
    user_id: i64,
    character_id: i64,
    waypoint: Waypoint,
) -> Result<()> {
    if !owns(pool, user_id, character_id).await? {
        return Err(invalid("that character isn't yours"));
    }
    validate_destination(pool, waypoint.destination_id).await?;
    send_waypoint(pool, &esi, character_id, waypoint).await
}

/// Set the waypoint for every online character of the user. Best-effort: a character
/// without the scope is skipped, and it fails only when none succeed.
pub async fn set_waypoint_all(
    pool: &PgPool,
    esi: Esi<'_>,
    user_id: i64,
    waypoint: Waypoint,
) -> Result<()> {
    validate_destination(pool, waypoint.destination_id).await?;
    let ids = sqlx::query_scalar!(
        "select c.id from characters c
         join character_status s on s.character_id = c.id and s.online
         where c.user_id = $1",
        user_id,
    )
    .fetch_all(pool)
    .await?;
    if ids.is_empty() {
        return Err(invalid("no characters online"));
    }
    let mut ok = 0usize;
    let mut last_err = None;
    for id in ids {
        match send_waypoint(pool, &esi, id, waypoint).await {
            Ok(()) => ok += 1,
            Err(err) => last_err = Some(err),
        }
    }
    match last_err {
        Some(err) if ok == 0 => Err(err),
        _ => Ok(()),
    }
}

/// ESI accepts a solar system, station, or structure id as a waypoint destination. We can
/// only vouch for the first two (structures are per-character ACL'd and not seeded), so an
/// unknown id is refused here rather than passed to ESI.
async fn validate_destination(pool: &PgPool, destination_id: i64) -> Result<()> {
    let exists = sqlx::query_scalar!(
        "select exists(select 1 from solar_systems where id = $1)
             or exists(select 1 from stations where id = $1)",
        destination_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);
    if exists {
        Ok(())
    } else {
        Err(invalid("unknown destination"))
    }
}

async fn send_waypoint(
    pool: &PgPool,
    esi: &Esi<'_>,
    character_id: i64,
    waypoint: Waypoint,
) -> Result<()> {
    let store = PgTokenStore::new(pool.clone());
    let token = esi
        .sso
        .access_token(&store, character_id, Scope::WriteWaypoint)
        .await
        .map_err(|err| match err {
            EsiError::MissingScope(_) => CharacterError::MissingScope,
            EsiError::TokenRejected => {
                invalid("EVE signed that character out, log in with it again")
            }
            other => CharacterError::Upstream(other),
        })?;
    esi.esi
        .set_waypoint(
            &token,
            waypoint.destination_id,
            waypoint.add_to_beginning,
            waypoint.clear_other_waypoints,
        )
        .await
        .map_err(CharacterError::Upstream)
}
