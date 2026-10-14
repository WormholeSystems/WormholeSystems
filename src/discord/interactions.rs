//! Discord interactions, delivered over HTTPS and verified by signature.
//!
//! Discord signs every interaction with the application's Ed25519 key over
//! `timestamp || body`, and that check is the whole security model: the endpoint is public,
//! and a badly-signed request must get a 401 or Discord marks the endpoint unhealthy. The
//! body is verified as raw bytes, before any JSON parsing, since the signature covers the
//! exact bytes sent.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::AppState;

/// Discord's interaction types, as far as this cares.
const PING: u8 = 1;
const APPLICATION_COMMAND: u8 = 2;
const AUTOCOMPLETE: u8 = 4;

/// And the replies.
const PONG: u8 = 1;
const CHANNEL_MESSAGE: u8 = 4;
const AUTOCOMPLETE_RESULT: u8 = 8;

/// Only the sender sees it. Everything WormholeSystems replies with is about the sender's own maps,
/// so it stays out of the channel.
const EPHEMERAL: u32 = 1 << 6;

/// `POST /discord/interactions`, the bot.
pub async fn handle(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    let Some(config) = state.discord.as_ref() else {
        return (StatusCode::NOT_FOUND, "Discord is not configured").into_response();
    };
    if !verify(&config.public_key, &headers, &body) {
        // Discord probes this with a deliberately bad signature and expects a 401.
        return (StatusCode::UNAUTHORIZED, "invalid request signature").into_response();
    }
    let Ok(interaction) = serde_json::from_slice::<Interaction>(&body) else {
        return (StatusCode::BAD_REQUEST, "unreadable interaction").into_response();
    };

    match interaction.kind {
        PING => axum::Json(json!({ "type": PONG })).into_response(),
        APPLICATION_COMMAND => {
            let text = super::commands::run(&state, &interaction).await;
            axum::Json(json!({
                "type": CHANNEL_MESSAGE,
                "data": { "content": text, "flags": EPHEMERAL }
            }))
            .into_response()
        }
        AUTOCOMPLETE => {
            let choices = super::commands::autocomplete(&state, &interaction).await;
            axum::Json(json!({
                "type": AUTOCOMPLETE_RESULT,
                "data": { "choices": choices }
            }))
            .into_response()
        }
        _ => (StatusCode::BAD_REQUEST, "unsupported interaction").into_response(),
    }
}

/// Whether this really came from Discord.
pub fn verify(public_key: &str, headers: &HeaderMap, body: &[u8]) -> bool {
    let Some(signature) = header(headers, "x-signature-ed25519") else {
        return false;
    };
    let Some(timestamp) = header(headers, "x-signature-timestamp") else {
        return false;
    };
    let (Ok(key), Ok(signature)) = (hex::decode(public_key), hex::decode(signature)) else {
        return false;
    };
    let (Ok(key), Ok(signature)) = (
        <[u8; 32]>::try_from(key.as_slice()),
        <[u8; 64]>::try_from(signature.as_slice()),
    ) else {
        return false;
    };
    let Ok(key) = VerifyingKey::from_bytes(&key) else {
        return false;
    };
    let mut message = timestamp.as_bytes().to_vec();
    message.extend_from_slice(body);
    key.verify_strict(&message, &Signature::from_bytes(&signature))
        .is_ok()
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

/// One interaction, narrowed to the fields the commands read.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Interaction {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(default)]
    pub data: Option<CommandData>,
    /// Present in a server, absent in a direct message.
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
    /// Set in a server; `user` is set instead in a DM.
    #[serde(default)]
    pub member: Option<Member>,
    #[serde(default)]
    pub user: Option<super::DiscordUser>,
}

impl Interaction {
    /// Who sent it, wherever they sent it from.
    pub fn sender(&self) -> Option<&super::DiscordUser> {
        self.member
            .as_ref()
            .and_then(|m| m.user.as_ref())
            .or(self.user.as_ref())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Member {
    #[serde(default)]
    pub user: Option<super::DiscordUser>,
    /// The sender's permissions in the channel, as Discord's decimal bitfield string.
    #[serde(default)]
    pub permissions: Option<String>,
}

/// The permission bits the alert commands ask about, from Discord's permission table.
#[derive(Debug, Clone, Copy)]
pub enum Permission {
    Administrator = 1 << 3,
    ManageChannels = 1 << 4,
    MentionEveryone = 1 << 17,
    ManageRoles = 1 << 28,
}

impl Member {
    /// Whether the sender holds the permission here. Administrators hold every one, which
    /// Discord does not spell out bit by bit.
    pub fn can(&self, permission: Permission) -> bool {
        let Some(bits) = self
            .permissions
            .as_deref()
            .and_then(|p| p.parse::<u64>().ok())
        else {
            return false;
        };
        bits & Permission::Administrator as u64 != 0 || bits & permission as u64 != 0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandData {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub options: Vec<CommandOption>,
    /// The objects behind role, user and channel options, keyed by id.
    #[serde(default)]
    pub resolved: Option<Resolved>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Resolved {
    #[serde(default)]
    pub roles: std::collections::HashMap<String, ResolvedRole>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResolvedRole {
    pub name: String,
}

impl CommandData {
    /// The name of a role option's role, when Discord sent it along.
    pub fn role_name(&self, role_id: &str) -> Option<&str> {
        self.resolved
            .as_ref()?
            .roles
            .get(role_id)
            .map(|role| role.name.as_str())
    }
}

/// An option, or a subcommand carrying its own. Discord nests them in the same shape.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandOption {
    pub name: String,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub options: Vec<CommandOption>,
    /// Set on the option the user is currently typing, during autocomplete.
    #[serde(default)]
    pub focused: bool,
}

impl CommandOption {
    pub fn string(&self) -> Option<&str> {
        self.value.as_ref()?.as_str()
    }

    pub fn integer(&self) -> Option<i64> {
        match self.value.as_ref()? {
            Value::Number(n) => n.as_i64(),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }
}

/// Find an option by name among a command's options.
pub fn option<'a>(options: &'a [CommandOption], name: &str) -> Option<&'a CommandOption> {
    options.iter().find(|o| o.name == name)
}

/// The option the user is typing, for autocomplete.
pub fn focused(options: &[CommandOption]) -> Option<&CommandOption> {
    options
        .iter()
        .find(|o| o.focused)
        .or_else(|| options.iter().find_map(|o| focused(&o.options)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    /// The key Discord publishes is hex; anything that is not is a misconfiguration, and
    /// must fail closed rather than skip the check.
    #[test]
    fn a_malformed_key_or_signature_never_verifies() {
        let mut headers = HeaderMap::new();
        headers.insert("x-signature-ed25519", HeaderValue::from_static("zz"));
        headers.insert("x-signature-timestamp", HeaderValue::from_static("1"));
        assert!(!verify("not-hex", &headers, b"{}"));
        assert!(!verify("00", &headers, b"{}"));
    }

    #[test]
    fn a_request_without_the_headers_never_verifies() {
        assert!(!verify("00", &HeaderMap::new(), b"{}"));
    }

    #[test]
    fn a_ping_parses_without_any_command_data() {
        let interaction: Interaction = serde_json::from_str(r#"{"type":1}"#).unwrap();
        assert_eq!(interaction.kind, PING);
        assert!(interaction.data.is_none());
    }

    #[test]
    fn the_sender_is_found_in_a_server_or_a_dm() {
        let in_guild: Interaction = serde_json::from_str(
            r#"{"type":2,"guild_id":"1","member":{"user":{"id":"42","username":"pilot"}}}"#,
        )
        .unwrap();
        assert_eq!(in_guild.sender().unwrap().id, "42");

        let in_dm: Interaction =
            serde_json::from_str(r#"{"type":2,"user":{"id":"42","username":"pilot"}}"#).unwrap();
        assert_eq!(in_dm.sender().unwrap().id, "42");
    }

    #[test]
    fn options_are_found_by_name_and_read_as_typed() {
        let data: CommandData = serde_json::from_str(
            r#"{"name":"route","options":[{"name":"map","value":7},{"name":"system","value":"30000142"}]}"#,
        )
        .unwrap();
        assert_eq!(option(&data.options, "map").unwrap().integer(), Some(7));
        assert_eq!(
            option(&data.options, "system").unwrap().integer(),
            Some(30000142)
        );
        assert!(option(&data.options, "missing").is_none());
    }

    /// Administrator is one bit that stands for all of them; an unreadable field is no
    /// permission at all rather than a guess.
    #[test]
    fn permissions_are_read_from_the_bitfield() {
        let member = |permissions: Option<&str>| Member {
            user: None,
            permissions: permissions.map(str::to_string),
        };
        let manage_channels = (Permission::ManageChannels as u64).to_string();
        assert!(member(Some(&manage_channels)).can(Permission::ManageChannels));
        assert!(!member(Some(&manage_channels)).can(Permission::ManageRoles));
        let administrator = (Permission::Administrator as u64).to_string();
        assert!(member(Some(&administrator)).can(Permission::MentionEveryone));
        assert!(!member(None).can(Permission::ManageChannels));
        assert!(!member(Some("lots")).can(Permission::ManageChannels));
    }

    #[test]
    fn a_role_option_is_named_from_the_resolved_objects() {
        let data: CommandData = serde_json::from_str(
            r#"{"name":"wh","options":[],"resolved":{"roles":{"77":{"name":"Scouts"}}}}"#,
        )
        .unwrap();
        assert_eq!(data.role_name("77"), Some("Scouts"));
        assert_eq!(data.role_name("78"), None);
    }

    /// Discord marks the focused option inside whichever subcommand it belongs to.
    #[test]
    fn the_focused_option_is_found_through_subcommands() {
        let data: CommandData = serde_json::from_str(
            r#"{"name":"alert","options":[{"name":"killmail","options":[
                 {"name":"map","value":"jit","focused":true}]}]}"#,
        )
        .unwrap();
        assert_eq!(focused(&data.options).unwrap().name, "map");
    }
}
