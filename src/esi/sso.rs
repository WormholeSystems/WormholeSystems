//! EVE SSO (OAuth 2.0 Authorization Code flow). The web layer generates and stores the
//! CSRF `state`, redirects to [`Sso::authorize_url`], then calls [`Sso::exchange_code`].
//! [`Sso::access_token`] resolves a usable token through a [`TokenStore`] with refresh.

use std::time::{Duration, Instant, SystemTime};

use jsonwebtoken::jwk::JwkSet;
use serde::Deserialize;
use tokio::sync::Mutex;

use super::scopes::Scope;
use super::token::{Token, TokenStore};
use super::{EsiError, Result, jwt};

const METADATA_URL: &str = "https://login.eveonline.com/.well-known/oauth-authorization-server";

/// How long the SSO's signing keys are reused before they are fetched again. Every login
/// and every token refresh validates a JWT, and the keys rarely change; a token signed by a
/// key we have not seen yet fetches them early regardless.
const JWKS_TTL: Duration = Duration::from_secs(60 * 60);

/// A token this close to expiry is refreshed before use rather than sent and refused.
pub const REFRESH_LEEWAY: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct SsoConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<Scope>,
}

impl std::fmt::Debug for SsoConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SsoConfig")
            .field("client_id", &self.client_id)
            .field("client_secret", &"<redacted>")
            .field("redirect_uri", &self.redirect_uri)
            .field("scopes", &self.scopes)
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
}

pub struct Sso {
    http: reqwest::Client,
    config: SsoConfig,
    metadata: Metadata,
    /// The signing keys and when they were fetched.
    jwks: Mutex<Option<(JwkSet, Instant)>>,
}

impl Sso {
    pub async fn discover(http: reqwest::Client, config: SsoConfig) -> Result<Sso> {
        let metadata: Metadata = http.get(METADATA_URL).send().await?.json().await?;
        Ok(Sso {
            http,
            config,
            metadata,
            jwks: Mutex::new(None),
        })
    }

    /// A non-functional SSO for the HTTP test harness: handlers under test never touch
    /// auth, but building an `AppState` still needs one. Any real use fails at the
    /// network, not before.
    pub fn stub(http: reqwest::Client, config: SsoConfig) -> Sso {
        Sso {
            http,
            config,
            metadata: Metadata {
                issuer: "test".into(),
                authorization_endpoint: "http://127.0.0.1:0/authorize".into(),
                token_endpoint: "http://127.0.0.1:0/token".into(),
                jwks_uri: "http://127.0.0.1:0/jwks".into(),
            },
            jwks: Mutex::new(None),
        }
    }

    /// `state` is the CSRF token the caller generates and verifies on the callback.
    pub fn authorize_url(&self, state: &str) -> String {
        self.authorize_url_for(state, &self.config.scopes)
    }

    /// Consent for an explicit scope set, for topping up a character's permissions.
    ///
    /// The caller is responsible for including everything the character already granted:
    /// SSO issues a token for exactly what is asked for, so consenting to one scope on its
    /// own would quietly drop the rest.
    pub fn authorize_url_for(&self, state: &str, scopes: &[Scope]) -> String {
        let scope = scopes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let params = [
            ("response_type", "code"),
            ("client_id", self.config.client_id.as_str()),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("scope", scope.as_str()),
            ("state", state),
        ];
        reqwest::Url::parse_with_params(&self.metadata.authorization_endpoint, params)
            .expect("authorization_endpoint is a valid base URL")
            .to_string()
    }

    pub async fn exchange_code(&self, code: &str) -> Result<(Token, jwt::Claims)> {
        let resp = self
            .token_request(&[("grant_type", "authorization_code"), ("code", code)])
            .await?;
        self.build_token(resp).await
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<(Token, jwt::Claims)> {
        let resp = self
            .token_request(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .await?;
        self.build_token(resp).await
    }

    /// Load `character_id`'s token, refresh + save it if it's expiring, and require
    /// `scope` before returning the access token.
    pub async fn access_token<S: TokenStore>(
        &self,
        store: &S,
        character_id: i64,
        scope: Scope,
    ) -> Result<String> {
        let token = self.stored_token(store, character_id).await?;
        if !token.has_scope(scope) {
            return Err(EsiError::MissingScope(scope.as_str().to_string()));
        }
        Ok(token.access_token)
    }

    /// Load `character_id`'s token, refreshing and saving it first if it is expiring.
    pub async fn stored_token<S: TokenStore>(&self, store: &S, character_id: i64) -> Result<Token> {
        let mut token = store
            .load(character_id)
            .await?
            .ok_or_else(|| EsiError::Auth(format!("no token for character {character_id}")))?;

        if token.expires_within(REFRESH_LEEWAY) {
            (token, _) = self.refresh(&token.refresh_token).await?;
            store.save(character_id, &token).await?;
        }
        Ok(token)
    }

    async fn token_request(&self, form: &[(&str, &str)]) -> Result<TokenResponse> {
        let resp = self
            .http
            .post(&self.metadata.token_endpoint)
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(form)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(EsiError::Auth(format!("token endpoint {status}: {body}")));
        }
        Ok(resp.json().await?)
    }

    async fn build_token(&self, resp: TokenResponse) -> Result<(Token, jwt::Claims)> {
        // Scopes come from the validated JWT, not the token response body.
        let jwks = self.jwks_for(&jwt::key_id(&resp.access_token)?).await?;
        let claims = jwt::validate(
            &jwks,
            &self.metadata.issuer,
            &self.config.client_id,
            &resp.access_token,
        )?;
        let token = Token {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_at: SystemTime::now() + Duration::from_secs(resp.expires_in),
            scopes: claims.scopes.clone(),
        };
        Ok((token, claims))
    }

    /// The SSO's signing keys, from the cache while they are fresh and include `kid`.
    async fn jwks_for(&self, kid: &str) -> Result<JwkSet> {
        let mut cached = self.jwks.lock().await;
        if let Some((keys, fetched_at)) = cached.as_ref()
            && fetched_at.elapsed() < JWKS_TTL
            && keys.find(kid).is_some()
        {
            return Ok(keys.clone());
        }
        let keys: JwkSet = self
            .http
            .get(&self.metadata.jwks_uri)
            .send()
            .await?
            .json()
            .await?;
        *cached = Some((keys.clone(), Instant::now()));
        Ok(keys)
    }
}
