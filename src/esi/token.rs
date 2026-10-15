use std::time::{Duration, SystemTime};

use super::Result;
use super::scopes::Scope;

#[derive(Debug, Clone)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: SystemTime,
    pub scopes: Vec<String>,
}

impl Token {
    /// Expired, or within `leeway` of expiring (so we refresh proactively).
    /// `expires_within(Duration::ZERO)` is "already expired".
    pub fn expires_within(&self, leeway: Duration) -> bool {
        SystemTime::now() + leeway >= self.expires_at
    }

    pub fn has_scope(&self, scope: Scope) -> bool {
        self.scopes.iter().any(|s| s == scope.as_str())
    }
}

/// Token persistence, implemented by the database layer so the ESI/SSO code never
/// depends on a concrete store.
#[allow(async_fn_in_trait)]
pub trait TokenStore {
    /// The character's token. A token EVE has refused is
    /// [`EsiError::TokenRejected`](super::EsiError::TokenRejected), not a token.
    async fn load(&self, character_id: i64) -> Result<Option<Token>>;
    async fn save(&self, character_id: i64, token: &Token) -> Result<()>;
    /// Remember that EVE refused the character's refresh token. `true` the first time, so
    /// the caller says so once.
    async fn reject(&self, character_id: i64) -> Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(scopes: &[Scope], expires_at: SystemTime) -> Token {
        Token {
            access_token: "a".into(),
            refresh_token: "r".into(),
            expires_at,
            scopes: scopes.iter().map(|s| s.as_str().to_string()).collect(),
        }
    }

    #[test]
    fn expiry() {
        let past = SystemTime::now() - Duration::from_secs(10);
        let future = SystemTime::now() + Duration::from_secs(3600);
        assert!(token(&[], past).expires_within(Duration::ZERO));
        assert!(!token(&[], future).expires_within(Duration::ZERO));
        assert!(token(&[], future).expires_within(Duration::from_secs(7200)));
        assert!(!token(&[], future).expires_within(Duration::from_secs(60)));
    }

    #[test]
    fn scopes() {
        let t = token(&[Scope::ReadLocation], SystemTime::now());
        assert!(t.has_scope(Scope::ReadLocation));
        assert!(!t.has_scope(Scope::WriteWaypoint));
    }
}
