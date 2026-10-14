//! Application configuration, read from the environment (and a `.env` file in dev).

use crate::allowlist::AffiliationAllowlist;
use crate::esi::{Scope, SsoConfig};
use crate::maps::GridConfig;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("{0}: {1}")]
    Invalid(&'static str, String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    /// Who is running this install, for the user agent every outbound request carries.
    /// Required: see [`crate::user_agent`].
    pub contact_name: String,
    pub contact_email: String,
    pub sso: SsoConfig,
    pub grid: GridConfig,
    /// Where ESI lives. Overridable so the e2e suite can point the whole stack at a stub
    /// and drive a pilot around without a live EVE session.
    pub esi_base_url: String,
    /// Absent when the application has no Discord app configured, which is the normal
    /// state for a dev machine: the alerts UI still works, the bot half simply is not there.
    pub discord: Option<DiscordConfig>,
    /// Where uploaded files (map background images) are kept. Relative to the working
    /// directory unless absolute; in the container it is a volume, so it outlives updates.
    pub uploads_dir: std::path::PathBuf,
    /// Who may log in at all. Empty, the default, lets in anyone with an EVE account.
    pub allowed_logins: AffiliationAllowlist,
    /// Who may create maps. Empty, the default, lets everyone who can log in create one.
    pub map_creators: AffiliationAllowlist,
}

/// What the Discord half of the integration needs.
///
/// All or nothing: a half-configured app fails in ways that look like bugs, so this is read
/// as one unit and left `None` unless every part is present.
#[derive(Debug, Clone)]
pub struct DiscordConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    /// Verifies that an interaction really came from Discord. Hex, from the app's page.
    pub public_key: String,
    /// Only needed to post as the bot; account linking and slash commands work without it.
    pub bot_token: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Config, ConfigError> {
        // Load `.env` if present; real environment variables take precedence.
        dotenvy::dotenv().ok();

        Ok(Config {
            database_url: required("DATABASE_URL")?,
            contact_name: required(crate::user_agent::NAME_VAR)?,
            contact_email: required(crate::user_agent::EMAIL_VAR)?,
            sso: SsoConfig {
                client_id: required("EVE_CLIENT_ID")?,
                client_secret: required("EVE_CLIENT_SECRET")?,
                redirect_uri: required("EVE_REDIRECT_URI")?,
                scopes: Scope::ALL.to_vec(),
            },
            grid: grid_from_env(),
            esi_base_url: std::env::var("ESI_BASE_URL")
                .unwrap_or_else(|_| crate::esi::BASE_URL.to_string()),
            discord: discord_from_env(),
            uploads_dir: nonempty("UPLOADS_DIR")
                .map(Into::into)
                .unwrap_or_else(|| std::path::PathBuf::from("data/uploads")),
            allowed_logins: allowlist("ALLOWED_AFFILIATION_IDS")?,
            map_creators: allowlist("MAP_CREATOR_AFFILIATION_IDS")?,
        })
    }
}

fn discord_from_env() -> Option<DiscordConfig> {
    Some(DiscordConfig {
        client_id: nonempty("DISCORD_CLIENT_ID")?,
        client_secret: nonempty("DISCORD_CLIENT_SECRET")?,
        redirect_uri: nonempty("DISCORD_REDIRECT_URI")?,
        public_key: nonempty("DISCORD_PUBLIC_KEY")?,
        bot_token: nonempty("DISCORD_BOT_TOKEN"),
    })
}

/// The value of `key`, or `None` when it is unset or blank.
fn nonempty(key: &str) -> Option<String> {
    let value = std::env::var(key).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// A variable set to nothing counts as missing. Compose passes an empty `.env` value
/// through as an empty string, and taking it at face value moves the failure from here,
/// where the message names the variable, to wherever the value is first used.
fn required(key: &'static str) -> Result<String, ConfigError> {
    nonempty(key).ok_or(ConfigError::Missing(key))
}

/// A comma-separated list of EVE ids, empty when unset.
fn allowlist(key: &'static str) -> Result<AffiliationAllowlist, ConfigError> {
    AffiliationAllowlist::parse(&std::env::var(key).unwrap_or_default())
        .map_err(|err| ConfigError::Invalid(key, err.to_string()))
}

/// Build the map [`GridConfig`] from the environment; each field falls back to its default
/// when the var is unset or unparseable.
fn grid_from_env() -> GridConfig {
    let d = GridConfig::default();
    GridConfig {
        cell_size: optional_f64("GRID_CELL_SIZE", d.cell_size),
        world_width: optional_f64("GRID_WORLD_WIDTH", d.world_width),
        world_height: optional_f64("GRID_WORLD_HEIGHT", d.world_height),
        viewport_height: optional_f64("GRID_VIEWPORT_HEIGHT", d.viewport_height),
    }
}

fn optional_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compose hands a blank `.env` value to the container as an empty string, which used to
    // pass for present and then panic on first use, several minutes into the first boot.
    #[test]
    fn a_variable_set_to_nothing_is_missing() {
        unsafe {
            std::env::set_var("WS_TEST_BLANK", "   ");
            std::env::set_var("WS_TEST_FILLED", " kept ");
        }
        assert!(nonempty("WS_TEST_BLANK").is_none());
        assert!(nonempty("WS_TEST_UNSET_ENTIRELY").is_none());
        assert_eq!(nonempty("WS_TEST_FILLED").as_deref(), Some("kept"));
    }
}
