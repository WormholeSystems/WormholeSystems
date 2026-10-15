use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::config::DatabaseConfig;
use crate::esi::token::{Token, TokenStore};
use crate::esi::{EsiError, Result as EsiResult};

/// How long anything waits for a free connection before giving up with an error. sqlx's
/// default is 30 seconds, which turns a saturated pool into requests that hang rather than
/// fail; a short wait surfaces the problem while it is happening.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(10);

/// Connect to Postgres and run any pending migrations.
pub async fn connect(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect(&config.url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

/// Postgres-backed [`TokenStore`].
#[derive(Clone)]
pub struct PgTokenStore {
    pool: PgPool,
}

impl PgTokenStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Lets the `TokenStore` impl use `?` on sqlx errors directly.
impl From<sqlx::Error> for EsiError {
    fn from(e: sqlx::Error) -> Self {
        EsiError::Store(e.to_string())
    }
}

impl TokenStore for PgTokenStore {
    async fn load(&self, character_id: i64) -> EsiResult<Option<Token>> {
        let row = sqlx::query!(
            "SELECT id, access_token, token_expires_at, refresh_token, rejected_at
             FROM esi_tokens WHERE character_id = $1
             ORDER BY updated_at DESC LIMIT 1",
            character_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else { return Ok(None) };
        if row.rejected_at.is_some() {
            return Err(EsiError::TokenRejected);
        }

        let scopes = sqlx::query_scalar!(
            "SELECT s.name FROM esi_token_scopes ts
             JOIN esi_scopes s ON s.id = ts.scope_id
             WHERE ts.token_id = $1",
            row.id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(Token {
            access_token: row.access_token.unwrap_or_default(),
            refresh_token: row.refresh_token,
            // A null/absent expiry is treated as already expired (forces a refresh).
            expires_at: row
                .token_expires_at
                .map(SystemTime::from)
                .unwrap_or(SystemTime::UNIX_EPOCH),
            scopes,
        }))
    }

    async fn save(&self, character_id: i64, token: &Token) -> EsiResult<()> {
        let expires_at = DateTime::<Utc>::from(token.expires_at);

        let mut tx = self.pool.begin().await?;

        // The store keeps one token per character; replace any existing one.
        sqlx::query!(
            "DELETE FROM esi_tokens WHERE character_id = $1",
            character_id
        )
        .execute(&mut *tx)
        .await?;

        let token_id = sqlx::query_scalar!(
            "INSERT INTO esi_tokens (character_id, access_token, token_expires_at, refresh_token)
             VALUES ($1, $2, $3, $4) RETURNING id",
            character_id,
            token.access_token,
            expires_at,
            token.refresh_token
        )
        .fetch_one(&mut *tx)
        .await?;

        // Every scope in one statement. `distinct`, because `do update` refuses to touch the
        // same row twice in one command; the no-op update is there so `returning` hands back
        // the ids of names that already existed too.
        sqlx::query!(
            "with scopes as (
                 insert into esi_scopes (name)
                 select distinct unnest($2::text[])
                 on conflict (name) do update set name = excluded.name
                 returning id
             )
             insert into esi_token_scopes (token_id, scope_id)
             select $1, id from scopes
             on conflict do nothing",
            token_id,
            &token.scopes,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn reject(&self, character_id: i64) -> EsiResult<bool> {
        let marked = sqlx::query!(
            "update esi_tokens set rejected_at = now()
             where character_id = $1 and rejected_at is null",
            character_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(marked > 0)
    }
}
