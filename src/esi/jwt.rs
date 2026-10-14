use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;

use super::{EsiError, Result};

#[derive(Debug, Clone)]
pub struct Claims {
    pub character_id: i64,
    pub name: String,
    /// Changes when the character is transferred to a different account.
    pub owner_hash: String,
    pub scopes: Vec<String>,
}

#[derive(Deserialize)]
struct RawClaims {
    sub: String,
    name: String,
    owner: String,
    #[serde(default)]
    scp: Option<Scp>,
}

/// `scp` is a single string for one scope, or an array for several.
#[derive(Deserialize)]
#[serde(untagged)]
enum Scp {
    One(String),
    Many(Vec<String>),
}

/// The id of the key that signed `token`, to pick it out of the SSO's key set.
pub fn key_id(token: &str) -> Result<String> {
    decode_header(token)
        .map_err(|e| EsiError::Jwt(e.to_string()))?
        .kid
        .ok_or_else(|| EsiError::Jwt("token has no `kid`".into()))
}

/// Validate signature (against the SSO's key set), issuer, audience, and expiry.
pub fn validate(jwks: &JwkSet, issuer: &str, client_id: &str, token: &str) -> Result<Claims> {
    let kid = key_id(token)?;
    let jwk = jwks
        .find(&kid)
        .ok_or_else(|| EsiError::Jwt(format!("no JWK for kid {kid}")))?;
    let key = DecodingKey::from_jwk(jwk).map_err(|e| EsiError::Jwt(e.to_string()))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer, "login.eveonline.com"]);
    validation.set_audience(&[client_id, "EVE Online"]);

    let raw = decode::<RawClaims>(token, &key, &validation)
        .map_err(|e| EsiError::Jwt(e.to_string()))?
        .claims;

    // sub is "CHARACTER:EVE:<id>".
    let character_id = raw
        .sub
        .rsplit(':')
        .next()
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| EsiError::Jwt(format!("unexpected sub: {}", raw.sub)))?;

    let scopes = match raw.scp {
        None => Vec::new(),
        Some(Scp::One(s)) => vec![s],
        Some(Scp::Many(v)) => v,
    };

    Ok(Claims {
        character_id,
        name: raw.name,
        owner_hash: raw.owner,
        scopes,
    })
}
