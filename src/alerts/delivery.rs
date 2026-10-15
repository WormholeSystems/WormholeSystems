//! Getting a message to Discord.
//!
//! Three ways in (a channel webhook, the bot posting in a channel, the bot direct-messaging
//! a linked user) sharing one embed, retry and rate-limit path.
//!
//! Discord rate-limits per webhook and answers 429 with `retry_after`. Honouring it is not
//! optional: ignoring it gets the whole application limited, not just the one webhook.

use std::time::Duration;

use serde::Serialize;

/// Colours from EVE's security bands, so an embed reads the way the map does.
pub fn security_color(security: f64) -> u32 {
    match crate::util::security::ccp_round_security(security) {
        s if s >= 0.9 => 0x2f_ef_ef,
        s if s >= 0.8 => 0x48_f0_c0,
        s if s >= 0.7 => 0x00_ef_47,
        s if s >= 0.6 => 0x00_f0_00,
        s if s >= 0.5 => 0x00_ff_00,
        s if s >= 0.4 => 0xd7_7700,
        s if s >= 0.3 => 0xf0_60_00,
        s if s >= 0.2 => 0xf0_48_00,
        s if s >= 0.1 => 0xd7_30_00,
        _ => 0xf0_00_f0,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub inline: bool,
}

impl Field {
    pub fn new(name: impl Into<String>, value: impl Into<String>, inline: bool) -> Field {
        Field {
            name: name.into(),
            value: value.into(),
            inline,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Embed {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub color: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Field>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<Footer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Image {
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Footer {
    pub text: String,
}

/// One message: the ping, if any, plus the embed that carries the detail.
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub embeds: Vec<Embed>,
    pub allowed_mentions: AllowedMentions,
}

/// Discord will ping anything mentioned in the text unless told otherwise, so every
/// message states exactly what it is allowed to reach.
#[derive(Debug, Clone, Serialize)]
pub struct AllowedMentions {
    pub parse: Vec<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,
}

impl Message {
    pub fn new(embed: Embed) -> Message {
        Message {
            content: None,
            embeds: vec![embed],
            allowed_mentions: AllowedMentions {
                parse: Vec::new(),
                roles: Vec::new(),
                users: Vec::new(),
            },
        }
    }

    /// Ping a role, and allow only that role to be pinged.
    pub fn mention_role(mut self, role_id: &str) -> Message {
        self.content = Some(format!("<@&{role_id}>"));
        self.allowed_mentions.roles = vec![role_id.to_string()];
        self
    }

    pub fn mention_user(mut self, user_id: &str) -> Message {
        self.content = Some(format!("<@{user_id}>"));
        self.allowed_mentions.users = vec![user_id.to_string()];
        self
    }

    pub fn mention_everyone(mut self) -> Message {
        self.content = Some("@everyone".to_string());
        self.allowed_mentions.parse = vec!["everyone"];
        self
    }
}

/// Why a send failed, and whether the alert should stop trying.
#[derive(Debug)]
pub enum SendError {
    /// The destination is gone: a deleted webhook or channel. Retrying will not help.
    Gone,
    /// Discord refused this message as malformed (400), with its explanation. The same
    /// message gets the same answer, so it is not sent again.
    Rejected(String),
    /// Anything else: network, 5xx, exhausted retries.
    Failed(String),
}

const ATTEMPTS: usize = 3;

/// How much of Discord's explanation for a refused message is kept. It names the field
/// that was wrong, which fits easily.
const REJECTION_CHARS: usize = 500;

/// One attempt at Discord, start to finish. The shared client only bounds a stalled read,
/// and a message held up for half a minute holds up the ones queued behind it.
const SEND_TIMEOUT: Duration = Duration::from_secs(15);

/// Post a message to a Discord webhook, honouring rate limits.
///
/// A 404 is a webhook somebody deleted, a 401 or 403 one whose token was rotated. Neither
/// comes back on its own, so the alert is told to stop rather than retry forever.
pub async fn post_webhook(
    http: &reqwest::Client,
    url: &str,
    message: &Message,
) -> Result<(), SendError> {
    send(http, http.post(url), message).await
}

/// Post as the bot into a channel it can see.
pub async fn post_channel(
    http: &reqwest::Client,
    token: &str,
    channel_id: &str,
    message: &Message,
) -> Result<(), SendError> {
    // Not a channel at all, so no retry will ever reach one.
    if !crate::discord::is_snowflake(channel_id) {
        return Err(SendError::Gone);
    }
    let url = format!("{}/channels/{channel_id}/messages", crate::discord::API);
    send(
        http,
        http.post(&url)
            .header("authorization", format!("Bot {token}")),
        message,
    )
    .await
}

/// Direct-message a Discord user.
///
/// Two calls: Discord has no "message this user" endpoint, only "open a channel with this
/// user" followed by the usual channel post. The channel id is stable, but caching it would
/// go stale when somebody blocks the bot, and the open call is cheap.
pub async fn post_dm(
    http: &reqwest::Client,
    token: &str,
    discord_user_id: &str,
    message: &Message,
) -> Result<(), SendError> {
    let opened = http
        .post(format!("{}/users/@me/channels", crate::discord::API))
        .timeout(SEND_TIMEOUT)
        .header("authorization", format!("Bot {token}"))
        .json(&serde_json::json!({ "recipient_id": discord_user_id }))
        .send()
        .await
        .map_err(|err| SendError::Failed(err.to_string()))?;
    if !opened.status().is_success() {
        // Most often: the recipient does not share a server with the bot, or has direct
        // messages closed. Neither fixes itself with a retry.
        return Err(SendError::Gone);
    }
    #[derive(serde::Deserialize)]
    struct Channel {
        id: String,
    }
    let channel: Channel = opened
        .json()
        .await
        .map_err(|err| SendError::Failed(err.to_string()))?;
    post_channel(http, token, &channel.id, message).await
}

/// The shared attempt loop: rate limits honoured, terminal failures reported as such.
async fn send(
    http: &reqwest::Client,
    request: reqwest::RequestBuilder,
    message: &Message,
) -> Result<(), SendError> {
    let mut last = String::new();
    for attempt in 0..ATTEMPTS {
        let Some(request) = request.try_clone() else {
            return Err(SendError::Failed("request is not retryable".into()));
        };
        let response = match request.timeout(SEND_TIMEOUT).json(message).send().await {
            Ok(response) => response,
            Err(err) => {
                last = err.to_string();
                tokio::time::sleep(backoff(attempt)).await;
                continue;
            }
        };
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        if matches!(status.as_u16(), 401 | 403 | 404) {
            return Err(SendError::Gone);
        }
        if status.as_u16() == 400 {
            let body = response.text().await.unwrap_or_default();
            return Err(SendError::Rejected(
                body.chars().take(REJECTION_CHARS).collect(),
            ));
        }
        if status.as_u16() == 429 {
            let wait = retry_after(&response).unwrap_or_else(|| backoff(attempt));
            last = format!("rate limited, waited {}ms", wait.as_millis());
            tokio::time::sleep(wait).await;
            continue;
        }
        last = format!("discord returned {status}");
        tokio::time::sleep(backoff(attempt)).await;
    }
    let _ = http;
    Err(SendError::Failed(last))
}

fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    let seconds: f64 = response
        .headers()
        .get("retry-after")?
        .to_str()
        .ok()?
        .parse()
        .ok()?;
    // A minute is already absurd for a webhook; anything beyond it is a bug or a ban, and
    // holding the task open for it helps nobody.
    Some(Duration::from_secs_f64(seconds.clamp(0.0, 60.0)))
}

fn backoff(attempt: usize) -> Duration {
    Duration::from_millis(250 * 2u64.pow(attempt as u32))
}

/// Fails unless every link an embed carries is absolute, the markdown ones in its text
/// included. Discord answers a relative one with a 400 and the message is lost.
#[cfg(test)]
pub(crate) fn assert_links_absolute(embed: &Embed) {
    let mut links: Vec<&str> = Vec::new();
    links.extend(embed.url.as_deref());
    links.extend(embed.thumbnail.as_ref().map(|i| i.url.as_str()));
    let texts = embed
        .description
        .iter()
        .chain(embed.fields.iter().map(|f| &f.value));
    for text in texts {
        let mut rest = text.as_str();
        while let Some(start) = rest.find("](") {
            let tail = &rest[start + 2..];
            let end = tail.find(')').unwrap_or(tail.len());
            links.push(&tail[..end]);
            rest = &tail[end..];
        }
    }
    for link in links {
        assert!(
            link.starts_with("https://") || link.starts_with("http://"),
            "{link} is not absolute"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_channel_id_that_is_not_a_snowflake_is_never_requested() {
        let message = Message::new(Embed::default());
        for id in [
            "",
            "../users/@me",
            "123/messages?x=",
            "12a",
            "123456789012345678901",
        ] {
            let sent = post_channel(&reqwest::Client::new(), "token", id, &message).await;
            assert!(matches!(sent, Err(SendError::Gone)), "{id:?}");
        }
    }

    #[test]
    fn security_colors_follow_the_bands() {
        assert_eq!(security_color(1.0), security_color(0.9));
        assert_ne!(security_color(0.5), security_color(0.4));
        // Wormholes and nullsec share the "no security" colour.
        assert_eq!(security_color(-0.99), security_color(0.0));
        // Banded on the displayed security, not the raw value.
        assert_eq!(security_color(0.4552), security_color(0.5));
        assert_eq!(security_color(0.02), security_color(0.1));
    }

    #[test]
    fn a_message_pings_only_what_it_names() {
        let message = Message::new(Embed::default()).mention_role("42");
        assert_eq!(message.content.as_deref(), Some("<@&42>"));
        assert_eq!(message.allowed_mentions.roles, vec!["42".to_string()]);
        assert!(message.allowed_mentions.parse.is_empty());
    }

    #[test]
    fn backoff_grows() {
        assert!(backoff(0) < backoff(1) && backoff(1) < backoff(2));
    }
}
