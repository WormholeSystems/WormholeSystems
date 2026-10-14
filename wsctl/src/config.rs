//! What the wizard collects, and what that turns into on disk.
//!
//! Separate from the prompting so the mapping can be tested without a terminal.

use std::collections::BTreeMap;

use rand::Rng;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Answers {
    pub setup: Setup,
    /// Full setup only. Empty means no domain: plain http on this machine.
    pub domain: String,
    pub http_port: u16,
    pub https_port: u16,
    pub contact_name: String,
    pub contact_email: String,
    pub eve_client_id: String,
    pub eve_client_secret: String,
    pub discord: Option<Discord>,
    /// Who may log in, as `ALLOWED_AFFILIATION_IDS` takes it. Empty lets everyone in.
    pub allowed_logins: String,
    /// Who may create maps, as `MAP_CREATOR_AFFILIATION_IDS` takes it. Empty lets everyone
    /// who can log in.
    pub map_creators: String,
}

/// How the install meets the world.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Setup {
    /// Its own Postgres and its own Caddy, which gets the certificate. One machine, nothing
    /// else on it needed.
    #[default]
    Full,
    /// Only the API and the frontend, on a machine that already has a Postgres and a reverse
    /// proxy.
    Standalone(Standalone),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Standalone {
    /// What the browser uses, scheme included. The proxy in front decides it, so it is
    /// asked for rather than derived.
    pub public_url: String,
    pub database_url: String,
    /// The Docker network the API and the frontend join, where a proxy container finds
    /// them as `wormholesystems-api` and `wormholesystems-web`.
    pub network: String,
    /// Loopback ports (API, frontend) for a proxy running on the machine itself rather than
    /// in Docker. None when the network is the only way in.
    pub host_ports: Option<(u16, u16)>,
}

/// The compose files each setup runs from. The full stack is compose's default file, so
/// it needs no COMPOSE_FILE at all, and an install from before there was a choice keeps
/// working untouched.
pub const STANDALONE_COMPOSE: &str = "compose.standalone.yml";
pub const STANDALONE_PORTS_COMPOSE: &str = "compose.standalone.ports.yml";

/// Everything only the standalone setup writes, struck out when an install goes back to
/// the full one so a stale COMPOSE_FILE cannot keep it standalone.
const STANDALONE_KEYS: [&str; 6] = [
    "COMPOSE_FILE",
    "WS_PUBLIC_URL",
    "WS_DATABASE_URL",
    "WS_PROXY_NETWORK",
    "WS_API_PORT",
    "WS_WEB_PORT",
];

impl Standalone {
    pub fn compose_file(&self) -> String {
        match self.host_ports {
            Some(_) => format!("{STANDALONE_COMPOSE}:{STANDALONE_PORTS_COMPOSE}"),
            None => STANDALONE_COMPOSE.to_string(),
        }
    }

    /// Read back what an earlier `setup` wrote. None for a full install.
    pub fn from_env(env: &str) -> Option<Self> {
        let get = |key: &str| crate::envfile::get(env, key).unwrap_or_default();
        if get("WS_SETUP") != "standalone" {
            return None;
        }
        let port = |key: &str, fallback: u16| get(key).parse().unwrap_or(fallback);
        let with_ports = get("COMPOSE_FILE").contains(STANDALONE_PORTS_COMPOSE);
        Some(Self {
            public_url: get("WS_PUBLIC_URL"),
            database_url: get("WS_DATABASE_URL"),
            network: Some(get("WS_PROXY_NETWORK"))
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| DEFAULT_NETWORK.to_string()),
            host_ports: with_ports.then(|| (port("WS_API_PORT", 3000), port("WS_WEB_PORT", 3001))),
        })
    }
}

pub const DEFAULT_NETWORK: &str = "wormholesystems";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Discord {
    pub application_id: String,
    pub public_key: String,
    pub client_id: String,
    pub client_secret: String,
    /// Only needed to post as the bot or send direct messages.
    pub bot_token: String,
}

impl Answers {
    /// What the outside world calls this install. Every redirect has to match it exactly.
    pub fn base_url(&self) -> String {
        if let Setup::Standalone(standalone) = &self.setup {
            return standalone.public_url.trim_end_matches('/').to_string();
        }
        if self.domain.is_empty() {
            format!("http://localhost:{}", self.http_port)
        } else {
            format!("https://{}", self.domain)
        }
    }

    /// The `.env` keys this produces. Discord is absent rather than blank when it is off:
    /// the server reads it all-or-nothing, and an empty client id counts as configured.
    pub fn env_values(&self) -> BTreeMap<String, String> {
        let mut values = BTreeMap::new();
        let mut set = |k: &str, v: String| {
            values.insert(k.to_string(), v);
        };
        match &self.setup {
            Setup::Full => {
                set("WS_SETUP", "full".into());
                set("WS_DOMAIN", self.domain.clone());
                set("HTTP_PORT", self.http_port.to_string());
                set("HTTPS_PORT", self.https_port.to_string());
            }
            // WS_DOMAIN and the Caddy ports stay as they were: nothing reads them here, and
            // going back to the full setup offers them as the defaults again.
            Setup::Standalone(standalone) => {
                set("WS_SETUP", "standalone".into());
                set("COMPOSE_FILE", standalone.compose_file());
                set("WS_PUBLIC_URL", self.base_url());
                set("WS_DATABASE_URL", standalone.database_url.clone());
                set("WS_PROXY_NETWORK", standalone.network.clone());
                if let Some((api, web)) = standalone.host_ports {
                    set("WS_API_PORT", api.to_string());
                    set("WS_WEB_PORT", web.to_string());
                }
            }
        }
        set("WS_CONTACT_NAME", self.contact_name.clone());
        set("WS_CONTACT_EMAIL", self.contact_email.clone());
        set("EVE_CLIENT_ID", self.eve_client_id.clone());
        set("EVE_CLIENT_SECRET", self.eve_client_secret.clone());
        set(
            "EVE_REDIRECT_URI",
            format!("{}/auth/callback", self.base_url()),
        );
        // Written even when empty: emptying it is how an instance is opened up again.
        set("ALLOWED_AFFILIATION_IDS", self.allowed_logins.clone());
        set("MAP_CREATOR_AFFILIATION_IDS", self.map_creators.clone());
        if let Some(discord) = &self.discord {
            set("DISCORD_APPLICATION_ID", discord.application_id.clone());
            set("DISCORD_PUBLIC_KEY", discord.public_key.clone());
            set("DISCORD_CLIENT_ID", discord.client_id.clone());
            set("DISCORD_CLIENT_SECRET", discord.client_secret.clone());
            set(
                "DISCORD_REDIRECT_URI",
                format!("{}/discord/callback", self.base_url()),
            );
            if !discord.bot_token.is_empty() {
                set("DISCORD_BOT_TOKEN", discord.bot_token.clone());
            }
        }
        values
    }

    /// The keys to strike out: Discord's after it was turned off, and whatever belongs to
    /// the setup this install is no longer using.
    pub fn env_removals(&self) -> Vec<&'static str> {
        let mut out = self.discord_removals();
        match &self.setup {
            Setup::Full => out.extend(STANDALONE_KEYS),
            Setup::Standalone(standalone) if standalone.host_ports.is_none() => {
                out.extend(["WS_API_PORT", "WS_WEB_PORT"])
            }
            Setup::Standalone(_) => {}
        }
        out
    }

    fn discord_removals(&self) -> Vec<&'static str> {
        const DISCORD: [&str; 6] = [
            "DISCORD_APPLICATION_ID",
            "DISCORD_PUBLIC_KEY",
            "DISCORD_CLIENT_ID",
            "DISCORD_CLIENT_SECRET",
            "DISCORD_REDIRECT_URI",
            "DISCORD_BOT_TOKEN",
        ];
        match &self.discord {
            None => DISCORD.to_vec(),
            Some(d) if d.bot_token.is_empty() => vec!["DISCORD_BOT_TOKEN"],
            Some(_) => Vec::new(),
        }
    }
}

/// A list of EVE ids as the server reads it: numbers, comma separated. Spaces and commas
/// both separate on the way in, since that is how a list gets pasted. Anything else is
/// refused here, because the server refuses to start on it.
pub fn affiliation_ids(input: &str) -> Result<String, String> {
    let mut ids = Vec::new();
    for entry in input.split([',', ' ']).filter(|e| !e.is_empty()) {
        match entry.parse::<u64>() {
            Ok(id) if id > 0 => ids.push(id.to_string()),
            _ => return Err(format!("{entry} is not an EVE id")),
        }
    }
    Ok(ids.join(","))
}

/// A password nobody has to think about. Alphanumeric so it survives a URL, a shell and a
/// compose file without escaping.
pub fn generated_password() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..32)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answers() -> Answers {
        Answers {
            setup: Setup::Full,
            domain: "map.example.com".into(),
            http_port: 80,
            https_port: 443,
            contact_name: "Nicolas Kion".into(),
            contact_email: "me@example.com".into(),
            eve_client_id: "abc".into(),
            eve_client_secret: "shh".into(),
            discord: None,
            allowed_logins: String::new(),
            map_creators: String::new(),
        }
    }

    #[test]
    fn affiliation_ids_are_tidied_into_what_the_server_reads() {
        assert_eq!(
            affiliation_ids(" 99000001, 98000002 2112000001,").unwrap(),
            "99000001,98000002,2112000001"
        );
        assert_eq!(affiliation_ids("").unwrap(), "");
        assert!(affiliation_ids("99000001,Goonswarm").is_err());
    }

    // An empty list has to be written as such, or a list emptied in the wizard would stay.
    #[test]
    fn the_allowlists_are_written_even_when_empty() {
        assert_eq!(answers().env_values()["ALLOWED_AFFILIATION_IDS"], "");
        assert_eq!(answers().env_values()["MAP_CREATOR_AFFILIATION_IDS"], "");
    }

    #[test]
    fn a_domain_means_https_and_no_domain_means_the_local_port() {
        assert_eq!(answers().base_url(), "https://map.example.com");
        let local = Answers {
            domain: String::new(),
            http_port: 8080,
            ..answers()
        };
        assert_eq!(local.base_url(), "http://localhost:8080");
    }

    // The EVE application rejects a callback that is not character-for-character the one
    // registered, so this is derived rather than typed a second time.
    #[test]
    fn the_callback_is_derived_from_the_base_url() {
        let values = answers().env_values();
        assert_eq!(
            values["EVE_REDIRECT_URI"],
            "https://map.example.com/auth/callback"
        );
    }

    #[test]
    fn discord_off_writes_nothing_and_strikes_the_keys_out() {
        let values = answers().env_values();
        assert!(!values.keys().any(|k| k.starts_with("DISCORD_")));
        assert!(answers().env_removals().contains(&"DISCORD_CLIENT_ID"));
    }

    #[test]
    fn discord_on_derives_its_own_callback() {
        let with = Answers {
            discord: Some(Discord {
                application_id: "1".into(),
                public_key: "k".into(),
                client_id: "2".into(),
                client_secret: "s".into(),
                bot_token: String::new(),
            }),
            ..answers()
        };
        let values = with.env_values();
        assert_eq!(
            values["DISCORD_REDIRECT_URI"],
            "https://map.example.com/discord/callback"
        );
        // No bot token given, so the key is removed rather than written empty.
        assert!(!values.contains_key("DISCORD_BOT_TOKEN"));
        assert_eq!(with.discord_removals(), vec!["DISCORD_BOT_TOKEN"]);
    }

    fn standalone(host_ports: Option<(u16, u16)>) -> Answers {
        Answers {
            setup: Setup::Standalone(Standalone {
                public_url: "https://map.example.com/".into(),
                database_url: "postgres://ws:secret@postgres:5432/ws".into(),
                network: "web".into(),
                host_ports,
            }),
            domain: String::new(),
            ..answers()
        }
    }

    // Behind somebody else's proxy the scheme is theirs to decide, so the URL is taken as
    // given (less a trailing slash) and every callback follows it.
    #[test]
    fn standalone_takes_the_public_url_as_given() {
        let values = standalone(None).env_values();
        assert_eq!(values["WS_PUBLIC_URL"], "https://map.example.com");
        assert_eq!(
            values["EVE_REDIRECT_URI"],
            "https://map.example.com/auth/callback"
        );
    }

    #[test]
    fn standalone_selects_its_compose_file_and_the_database() {
        let values = standalone(None).env_values();
        assert_eq!(values["WS_SETUP"], "standalone");
        assert_eq!(values["COMPOSE_FILE"], "compose.standalone.yml");
        assert_eq!(
            values["WS_DATABASE_URL"],
            "postgres://ws:secret@postgres:5432/ws"
        );
        assert_eq!(values["WS_PROXY_NETWORK"], "web");
        // The Caddy settings are left alone rather than rewritten for nothing.
        assert!(!values.contains_key("WS_DOMAIN"));
        assert!(!values.contains_key("WS_API_PORT"));
        assert!(standalone(None).env_removals().contains(&"WS_API_PORT"));
    }

    #[test]
    fn host_ports_add_the_ports_file() {
        let values = standalone(Some((4000, 4001))).env_values();
        assert_eq!(
            values["COMPOSE_FILE"],
            "compose.standalone.yml:compose.standalone.ports.yml"
        );
        assert_eq!(values["WS_API_PORT"], "4000");
        assert_eq!(values["WS_WEB_PORT"], "4001");
    }

    // A COMPOSE_FILE left behind would keep running the standalone stack after going back.
    #[test]
    fn going_back_to_full_strikes_out_everything_standalone() {
        let removals = answers().env_removals();
        for key in STANDALONE_KEYS {
            assert!(removals.contains(&key), "{key} stays behind");
        }
        assert_eq!(answers().env_values()["WS_SETUP"], "full");
    }

    #[test]
    fn what_setup_wrote_reads_back() {
        let written = standalone(Some((4000, 4001)));
        let env = crate::envfile::patch("", &written.env_values());
        let Setup::Standalone(expected) = written.setup else {
            unreachable!()
        };
        let read = Standalone::from_env(&env).unwrap();
        assert_eq!(read.database_url, expected.database_url);
        assert_eq!(read.network, expected.network);
        assert_eq!(read.host_ports, Some((4000, 4001)));
        assert_eq!(read.public_url, "https://map.example.com");
    }

    #[test]
    fn an_install_without_a_choice_is_full() {
        assert_eq!(Standalone::from_env("WS_DOMAIN=map.example.com\n"), None);
    }

    #[test]
    fn the_generated_password_needs_no_escaping() {
        let password = generated_password();
        assert_eq!(password.len(), 32);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
