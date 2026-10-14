//! Authorization helpers and the access-management actions (`set_access`,
//! `revoke_access`). See [access.md](../../docs/database/access.md).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::error::{MapError, Result};
use super::{Actor, Role, SubjectType};

/// The user's effective role on a map: the highest role they match across *all* their
/// characters (a character's own id, its corporation, or its alliance). `None` means no
/// access at all. Generic over the executor so the same predicate serves both a pool and
/// an open transaction.
pub async fn effective_role<'e, E>(executor: E, map_id: i64, user_id: i64) -> Result<Option<Role>>
where
    E: sqlx::PgExecutor<'e>,
{
    let roles = sqlx::query_scalar!(
        r#"select role
           from map_access
           where map_id = $1
             and (expires_at is null or expires_at > now())
             and subject_id in (
                 select id from characters where user_id = $2
                 union all
                 select corporation_id from characters where user_id = $2
                 union all
                 select alliance_id from characters where user_id = $2 and alliance_id is not null
             )"#,
        map_id,
        user_id,
    )
    .fetch_all(executor)
    .await?;

    Ok(roles.into_iter().max())
}

/// The role gap as the canonical errors: no access at all is `NotFound` so a map is not
/// revealed by refusing it; access but too low is `Forbidden`.
fn require(role: Option<Role>, min: Role) -> Result<Role> {
    match role {
        None => Err(MapError::NotFound),
        Some(role) if role >= min => Ok(role),
        Some(_) => Err(MapError::Forbidden),
    }
}

/// Delete grants that have run out. The view keeps an expired grant from counting the
/// moment it expires; this is what stops the rows accumulating, and is why the two exist
/// together rather than either being enough on its own.
pub async fn sweep_expired(pool: &PgPool) -> Result<u64> {
    Ok(
        sqlx::query!("delete from map_access where expires_at is not null and expires_at <= now()")
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

/// Run [`sweep_expired`] hourly. Hourly rather than daily because an expired grant is a
/// row somebody meant to be gone, and rather than per-minute because the view already makes
/// it stop counting — this is tidying, not enforcement.
pub fn spawn_expiry_sweep(pool: PgPool) {
    crate::background::spawn_every(
        "access expiry sweep",
        std::time::Duration::from_secs(60 * 60),
        move || {
            let pool = pool.clone();
            async move {
                let removed = sweep_expired(&pool).await?;
                if removed > 0 {
                    tracing::info!("access: {removed} expired grant(s) removed");
                }
                Ok::<_, crate::maps::MapError>(())
            }
        },
    );
}

/// Whether this user can see the map at all. The yes/no form of [`effective_role`], for the
/// callers that have nothing useful to say about *why* not: an alert deciding whether its
/// creator still belongs, or the bot deciding whether to list a map.
pub async fn can_see(pool: &PgPool, map_id: i64, user_id: i64) -> bool {
    effective_role(pool, map_id, user_id)
        .await
        .ok()
        .flatten()
        .is_some()
}

/// Require at least `min` on the map, returning the actor's actual role (callers use it as
/// the ceiling on what they may grant).
pub async fn require_role(pool: &PgPool, map_id: i64, user_id: i64, min: Role) -> Result<Role> {
    require(effective_role(pool, map_id, user_id).await?, min)
}

/// Who is looking at a map, for the read paths. A guest (no grant, reaching a shared map)
/// is a viewer and nothing more: no pilots, no history, no settings, no writes.
#[derive(Debug, Clone, Copy)]
pub struct Reader {
    /// The signed-in user, when there is one. `None` is a guest with a link.
    pub actor: Option<Actor>,
    pub role: Role,
}

/// How a map was opened to the world, if it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sharing {
    Private,
    /// Anyone with the address; no token needed.
    Public,
    /// Anyone holding the token in the query string.
    Token,
}

/// Resolve who may read this map: a grant first, then whatever the map has been opened up
/// to. `NotFound` rather than `Forbidden` when nothing lets them in, so a private map does
/// not confirm its own existence to a stranger.
pub async fn reader_for(
    pool: &PgPool,
    map_id: i64,
    actor: Option<Actor>,
    share_token: Option<&str>,
) -> Result<Reader> {
    if let Some(actor) = actor
        && let Some(role) = effective_role(pool, map_id, actor.user_id).await?
    {
        return Ok(Reader {
            actor: Some(actor),
            role,
        });
    }

    let map = sqlx::query!(
        "select is_public, share_token from maps where id = $1",
        map_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(MapError::NotFound)?;

    let shared = map.is_public
        || match (map.share_token.as_deref(), share_token) {
            // A blank token in the URL must never match a map that has none.
            (Some(theirs), Some(given)) => !theirs.is_empty() && theirs == given,
            _ => false,
        };
    if !shared {
        return Err(MapError::NotFound);
    }
    Ok(Reader {
        actor,
        role: Role::Viewer,
    })
}

/// As [`require_role`], but inside an open transaction so the check and the write it
/// guards commit (or roll back) together. Used by the command dispatcher.
pub(super) async fn require_role_tx(
    tx: &mut super::command::Tx<'_>,
    map_id: i64,
    user_id: i64,
    min: Role,
) -> Result<Role> {
    require(effective_role(&mut **tx, map_id, user_id).await?, min)
}

pub(super) async fn owns_character(pool: &PgPool, user_id: i64, character_id: i64) -> Result<bool> {
    let exists = sqlx::query_scalar!(
        "select exists(select 1 from characters where id = $1 and user_id = $2)",
        character_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists.unwrap_or(false))
}

/// One grant, with the subject's name resolved for display. `name` is `None` when the
/// subject is an entity we have never cached (a corp nobody on this map belongs to).
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AccessEntry {
    pub subject_type: SubjectType,
    pub subject_id: i64,
    pub name: Option<String>,
    pub role: Role,
    /// When the grant lapses. `None` lasts until someone revokes it.
    #[ts(optional)]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Every grant on the map, owners first. Viewer+.
pub async fn list_access(pool: &PgPool, actor: Actor, map_id: i64) -> Result<Vec<AccessEntry>> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;
    let entries = sqlx::query_as!(
        AccessEntry,
        r#"select a.subject_type,
                  a.subject_id,
                  coalesce(c.name, corp.name, al.name) as "name?",
                  a.role,
                  a.expires_at
           from map_access a
           left join characters c
             on a.subject_type = 'character' and c.id = a.subject_id
           left join corporations corp
             on a.subject_type = 'corporation' and corp.id = a.subject_id
           left join alliances al
             on a.subject_type = 'alliance' and al.id = a.subject_id
           where a.map_id = $1
             and (a.expires_at is null or a.expires_at > now())
           order by
             case a.role
               when 'owner' then 0 when 'manager' then 1
               when 'member' then 2 else 3
             end,
             coalesce(c.name, corp.name, al.name)"#,
        map_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(entries)
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetAccess {
    pub map_id: i64,
    pub subject_type: SubjectType,
    pub subject_id: i64,
    pub role: Role,
    /// When it should lapse. Absent leaves an existing expiry alone; `null` makes the
    /// grant permanent again.
    #[serde(default, deserialize_with = "super::double_option")]
    #[ts(optional)]
    pub expires_at: Option<Option<DateTime<Utc>>>,
}

/// Grant `role` to a subject, or change an existing subject's role. Manager+, and you
/// may not grant a role above your own. Never leaves the map without an owner.
pub async fn set_access(pool: &PgPool, actor: Actor, cmd: SetAccess) -> Result<()> {
    let actor_role = require_role(pool, cmd.map_id, actor.user_id, Role::Manager).await?;
    if cmd.role > actor_role {
        return Err(MapError::Forbidden);
    }
    // A map has exactly one owner; moving it is `transfer_ownership`, not a grant.
    if cmd.role == Role::Owner {
        return Err(MapError::Validation(
            "ownership is transferred, not granted".into(),
        ));
    }

    let mut tx = pool.begin().await?;
    sqlx::query!(
        "insert into map_access (map_id, subject_type, subject_id, role, expires_at)
         values ($1, $2, $3, $4, $5)
         on conflict (map_id, subject_id)
         do update set subject_type = excluded.subject_type, role = excluded.role,
             -- $6 = was expires_at present at all: absent leaves the existing expiry,
             -- a value (null included) replaces it.
             expires_at = case when $6 then $5 else map_access.expires_at end",
        cmd.map_id,
        cmd.subject_type,
        cmd.subject_id,
        cmd.role,
        cmd.expires_at.flatten(),
        cmd.expires_at.is_some(),
    )
    .execute(&mut *tx)
    .await?;
    ensure_has_owner(&mut tx, cmd.map_id).await?;
    tx.commit().await?;
    // Published here, not in the handler, so no caller can forget the event.
    crate::realtime::hub().publish(crate::realtime::MapEvent::AccessChanged { map_id: cmd.map_id });
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RevokeAccess {
    pub map_id: i64,
    pub subject_id: i64,
}

/// Remove a subject's grant. Manager+. Revoking the last owner is rejected.
pub async fn revoke_access(pool: &PgPool, actor: Actor, cmd: RevokeAccess) -> Result<()> {
    require_role(pool, cmd.map_id, actor.user_id, Role::Manager).await?;

    let mut tx = pool.begin().await?;
    let deleted = sqlx::query!(
        "delete from map_access where map_id = $1 and subject_id = $2",
        cmd.map_id,
        cmd.subject_id,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if deleted == 0 {
        return Err(MapError::NotFound);
    }
    ensure_has_owner(&mut tx, cmd.map_id).await?;
    tx.commit().await?;
    // Published here, not in the handler, so no caller can forget the event.
    crate::realtime::hub().publish(crate::realtime::MapEvent::AccessChanged { map_id: cmd.map_id });
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TransferOwnership {
    pub map_id: i64,
    /// The character taking it on. They keep no second role: there is one owner.
    pub subject_id: i64,
}

/// Hand the map to someone else. Owner only. The new owner must already be a character
/// with a grant on the map, and the old owner stays on as a manager.
pub async fn transfer_ownership(pool: &PgPool, actor: Actor, cmd: TransferOwnership) -> Result<()> {
    require_role(pool, cmd.map_id, actor.user_id, Role::Owner).await?;

    let mut tx = pool.begin().await?;
    let target = sqlx::query_scalar!(
        r#"select subject_type from map_access
           where map_id = $1 and subject_id = $2
             and (expires_at is null or expires_at > now())"#,
        cmd.map_id,
        cmd.subject_id,
    )
    .fetch_optional(&mut *tx)
    .await?;
    match target {
        Some(SubjectType::Character) => {}
        Some(_) => {
            return Err(MapError::Validation(
                "only a character can own a map".into(),
            ));
        }
        None => return Err(MapError::NotFound),
    }

    sqlx::query!(
        "update map_access set role = 'manager'
         where map_id = $1 and role = 'owner' and subject_id <> $2",
        cmd.map_id,
        cmd.subject_id,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "update map_access set role = 'owner', expires_at = null
         where map_id = $1 and subject_id = $2",
        cmd.map_id,
        cmd.subject_id,
    )
    .execute(&mut *tx)
    .await?;
    ensure_has_owner(&mut tx, cmd.map_id).await?;
    tx.commit().await?;
    // Published here, not in the handler, so no caller can forget the event.
    crate::realtime::hub().publish(crate::realtime::MapEvent::AccessChanged { map_id: cmd.map_id });
    Ok(())
}

/// Fail (rolling back the transaction) if the map has no owner left.
async fn ensure_has_owner(tx: &mut sqlx::PgConnection, map_id: i64) -> Result<()> {
    let owners = sqlx::query_scalar!(
        "select count(*) from map_access where map_id = $1 and role = 'owner'",
        map_id,
    )
    .fetch_one(&mut *tx)
    .await?
    .unwrap_or(0);
    if owners == 0 {
        return Err(MapError::LastOwner);
    }
    Ok(())
}
