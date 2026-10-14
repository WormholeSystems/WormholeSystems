//! The single-use `state` behind an OAuth handshake, for both the EVE login and linking a
//! Discord account.
//!
//! Each flow is stamped with the kind that started it and only that kind's callback can
//! consume it. Otherwise a state minted for one provider would be accepted by the other's
//! callback, which is exactly the cross-flow confusion the `state` is there to prevent.
//! See [`oauth_login_flows`](../docs/database/authentication.md#oauth_login_flows).

use sqlx::PgPool;
use uuid::Uuid;

/// Which handshake a state belongs to. Stored as the `kind` column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowKind {
    /// Signing in with EVE, or adding a character to the signed-in account.
    EveLogin,
    /// Linking the signed-in account to a Discord account.
    DiscordLink,
}

impl FlowKind {
    fn as_str(self) -> &'static str {
        match self {
            FlowKind::EveLogin => "eve_login",
            FlowKind::DiscordLink => "discord_link",
        }
    }
}

/// What the starting request left for the callback.
#[derive(Debug)]
pub struct Flow {
    pub link_user_id: Option<i64>,
    pub redirect_to: Option<String>,
}

/// Record a new handshake and return its `state`, good for ten minutes.
///
/// Anyone who starts a handshake and never comes back leaves a row behind, so the expired
/// ones are cleared here. The table only ever holds handshakes in flight, which keeps this
/// cheap enough to do every time rather than on a schedule.
pub async fn start(
    pool: &PgPool,
    kind: FlowKind,
    link_user_id: Option<i64>,
    redirect_to: Option<&str>,
) -> sqlx::Result<String> {
    sqlx::query!("delete from oauth_login_flows where expires_at <= now()")
        .execute(pool)
        .await?;
    let state = Uuid::new_v4().to_string();
    sqlx::query!(
        "insert into oauth_login_flows (state, kind, link_user_id, redirect_to, expires_at)
         values ($1, $2, $3, $4, now() + interval '10 minutes')",
        state,
        kind.as_str(),
        link_user_id,
        redirect_to,
    )
    .execute(pool)
    .await?;
    Ok(state)
}

/// Consume a `state` for a callback of `kind`. `None` when it is unknown, expired, already
/// used, or was started by the other kind of handshake. A state of the wrong kind is left
/// where it is, so a stray request cannot burn somebody else's login.
pub async fn consume(pool: &PgPool, kind: FlowKind, state: &str) -> sqlx::Result<Option<Flow>> {
    let row = sqlx::query!(
        "delete from oauth_login_flows
         where state = $1 and kind = $2 and expires_at > now()
         returning link_user_id, redirect_to",
        state,
        kind.as_str(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| Flow {
        link_user_id: r.link_user_id,
        redirect_to: r.redirect_to,
    }))
}
