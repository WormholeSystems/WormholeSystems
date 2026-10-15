//! Application configuration, read from the environment (and a `.env` file in dev).
//!
//! Every variable the server reads is read here, once, at startup. The subcommands that
//! need only a part of it (a database, the contact for outbound requests) read just that
//! section, through the same function the full [`Config`] uses.

use std::time::Duration;

use crate::allowlist::AffiliationAllowlist;
use crate::announcement::{Announcement, AnnouncementVars};
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
    pub database: DatabaseConfig,
    /// Where the HTTP server listens. Loopback by default; the container sets `0.0.0.0`.
    pub listen_addr: String,
    pub contact: Contact,
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
    /// The banner above every page, when the operator set one.
    pub announcement: Option<Announcement>,
    pub polling: PollingConfig,
    pub killmails: KillmailConfig,
    /// Where EVE Scout's public signatures are read from. Overridable so the e2e suite can
    /// serve a fixed set: the real list is scouted by hand and changes under the tests.
    pub eve_scout_url: String,
    /// Where people open the app, without a trailing slash, for links that leave it: a
    /// Discord embed pointing back at the map has to be absolute.
    pub public_url: String,
    /// The browser's error-reporting DSN, which the Sentry tunnel forwards to and nowhere
    /// else. Unset turns the tunnel off.
    pub web_sentry_dsn: Option<String>,
    /// Where Prometheus metrics are served, on a listener of their own. Unset is off.
    pub metrics_addr: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    /// How many connections the pool holds open at most.
    pub max_connections: u32,
}

/// Who is running this install, for the user agent every outbound request carries.
/// Required: see [`crate::user_agent`].
#[derive(Debug, Clone)]
pub struct Contact {
    pub name: String,
    pub email: String,
}

/// How often the ESI mirrors ask. The e2e stack tightens both so a test does not wait a
/// minute for a status change.
#[derive(Debug, Clone)]
pub struct PollingConfig {
    pub server_status: Duration,
    pub skyhooks: Duration,
}

/// The zKillboard feed and the archive backfill that goes with it.
#[derive(Debug, Clone)]
pub struct KillmailConfig {
    /// Off unless `ZKB_LISTEN=1`: the ingest, the name resolver, the backfill and the daily
    /// analysis all hang off it.
    pub listen: bool,
    pub base_url: String,
    /// How much history a boot fills in from EVE Ref's archives; 0 turns it off.
    pub backfill_days: u32,
}

/// What `discord-register` needs, which is not what the running bot needs: the application
/// id is only used to upload the commands.
#[derive(Debug, Clone)]
pub struct DiscordRegistration {
    pub application_id: String,
    pub bot_token: String,
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

        let redirect_uri = required("EVE_REDIRECT_URI")?;
        Ok(Config {
            database: DatabaseConfig::from_env()?,
            listen_addr: nonempty("LISTEN_ADDR").unwrap_or_else(|| "127.0.0.1:3000".to_string()),
            contact: Contact::from_env()?,
            sso: SsoConfig {
                client_id: required("EVE_CLIENT_ID")?,
                client_secret: required("EVE_CLIENT_SECRET")?,
                redirect_uri: redirect_uri.clone(),
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
            announcement: announcement_from_env(),
            polling: PollingConfig {
                server_status: seconds("SERVER_STATUS_POLL_SECS", 60),
                skyhooks: seconds("SKYHOOK_POLL_SECS", 5 * 60),
            },
            killmails: KillmailConfig {
                listen: std::env::var("ZKB_LISTEN").as_deref() == Ok("1"),
                base_url: nonempty("ZKB_BASE_URL")
                    .unwrap_or_else(|| crate::killmails::ZKB_BASE_URL.to_string()),
                backfill_days: std::env::var("KILLMAIL_BACKFILL_DAYS")
                    .ok()
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(crate::killmails::BACKFILL_DAYS),
            },
            eve_scout_url: nonempty("EVE_SCOUT_URL")
                .unwrap_or_else(|| "https://api.eve-scout.com/v2/public/signatures".to_string()),
            public_url: public_url(nonempty("WS_PUBLIC_URL").as_deref(), &redirect_uri)?,
            web_sentry_dsn: nonempty("PUBLIC_SENTRY_DSN"),
            metrics_addr: nonempty("METRICS_ADDR"),
        })
    }
}

impl DatabaseConfig {
    /// The default is not sqlx's five: the API, every open websocket, the tracking poller,
    /// killmail ingest and the alert lifecycle all draw on one pool, and the poller alone
    /// asks for more than five at a time, so five means requests queue behind background
    /// work rather than being served.
    pub const DEFAULT_MAX_CONNECTIONS: u32 = 20;

    pub fn from_env() -> Result<DatabaseConfig, ConfigError> {
        dotenvy::dotenv().ok();
        Ok(DatabaseConfig {
            url: required("DATABASE_URL")?,
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(Self::DEFAULT_MAX_CONNECTIONS),
        })
    }
}

impl Contact {
    pub const NAME_VAR: &str = "WS_CONTACT_NAME";
    pub const EMAIL_VAR: &str = "WS_CONTACT_EMAIL";

    pub fn from_env() -> Result<Contact, ConfigError> {
        dotenvy::dotenv().ok();
        Ok(Contact {
            name: required(Self::NAME_VAR)?,
            email: required(Self::EMAIL_VAR)?,
        })
    }
}

impl DiscordRegistration {
    pub fn from_env() -> Result<DiscordRegistration, ConfigError> {
        dotenvy::dotenv().ok();
        Ok(DiscordRegistration {
            application_id: required("DISCORD_APPLICATION_ID")?,
            bot_token: required("DISCORD_BOT_TOKEN")?,
        })
    }
}

/// A period in whole seconds, at least one.
fn seconds(key: &str, default: u64) -> Duration {
    let secs = std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default);
    Duration::from_secs(secs.max(1))
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

/// `WS_PUBLIC_URL` when it is set. Otherwise the EVE callback says it: that is the browser's
/// origin plus `/auth/callback`, or the SSO login would not come back.
fn public_url(explicit: Option<&str>, redirect_uri: &str) -> Result<String, ConfigError> {
    let (key, raw): (&'static str, &str) = match explicit {
        Some(url) => ("WS_PUBLIC_URL", url),
        None => (
            "EVE_REDIRECT_URI",
            redirect_uri
                .strip_suffix("/auth/callback")
                .unwrap_or(redirect_uri),
        ),
    };
    let parsed =
        reqwest::Url::parse(raw).map_err(|err| ConfigError::Invalid(key, err.to_string()))?;
    if !matches!(parsed.scheme(), "http" | "https") || !parsed.has_host() {
        return Err(ConfigError::Invalid(key, "not an http(s) address".into()));
    }
    // Only a callback that is not the usual one is cut back to its origin; anything else
    // keeps its path, so an install under a prefix links under it.
    let base = if explicit.is_none() && raw == redirect_uri {
        parsed.origin().ascii_serialization()
    } else {
        raw.to_string()
    };
    Ok(base.trim_end_matches('/').to_string())
}

fn announcement_from_env() -> Option<Announcement> {
    let var = |key: &str| std::env::var(key).ok();
    let (level, title, message, link_url, link_label, dismissible) = (
        var("ANNOUNCEMENT_LEVEL"),
        var("ANNOUNCEMENT_TITLE"),
        var("ANNOUNCEMENT_MESSAGE"),
        var("ANNOUNCEMENT_LINK_URL"),
        var("ANNOUNCEMENT_LINK_LABEL"),
        var("ANNOUNCEMENT_DISMISSIBLE"),
    );
    Announcement::from_vars(AnnouncementVars {
        level: level.as_deref(),
        title: title.as_deref(),
        message: message.as_deref(),
        link_url: link_url.as_deref(),
        link_label: link_label.as_deref(),
        dismissible: dismissible.as_deref(),
    })
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

    #[test]
    fn the_public_url_comes_from_the_setting_or_the_callback() {
        let callback = "https://map.example.com/auth/callback";
        assert_eq!(
            public_url(None, callback).unwrap(),
            "https://map.example.com"
        );
        assert_eq!(
            public_url(None, "http://localhost:5173/wh/auth/callback").unwrap(),
            "http://localhost:5173/wh"
        );
        assert_eq!(
            public_url(None, "https://map.example.com/sso").unwrap(),
            "https://map.example.com"
        );
        assert_eq!(
            public_url(Some("https://other.example.com/"), callback).unwrap(),
            "https://other.example.com"
        );
        assert!(public_url(Some("map.example.com"), callback).is_err());
    }
}
