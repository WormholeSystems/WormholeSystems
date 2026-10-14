//! What the wizard collects, and what that turns into on disk.
//!
//! Separate from the prompting so the mapping can be tested without a terminal.

use std::collections::BTreeMap;

use rand::Rng;

use crate::envfile;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Answers {
    pub deployment: Deployment,
    pub images: Images,
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

/// Where the API and frontend images come from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Images {
    /// The ones CI publishes for every push to main.
    #[default]
    Pull,
    /// Compiled here from the checkout, which is what every install did before there were
    /// published ones.
    Build,
}

impl Images {
    pub fn as_str(self) -> &'static str {
        match self {
            Images::Pull => "pull",
            Images::Build => "build",
        }
    }

    /// What an install runs from. One that never chose builds, since that is what it has
    /// always done; it keeps doing so until its operator opts in.
    pub fn from_env(env: &str) -> Self {
        match envfile::get(env, "WS_IMAGES").as_deref() {
            Some("pull") => Images::Pull,
            _ => Images::Build,
        }
    }

    /// Whether `.env` says at all, as opposed to falling back to building.
    pub fn chosen(env: &str) -> bool {
        envfile::get(env, "WS_IMAGES").is_some_and(|v| !v.is_empty())
    }
}

/// How the install is put together: two choices that used to be one ("full" ran both
/// bundled, "standalone" neither).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Deployment {
    pub database: Database,
    pub proxy: Proxy,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Database {
    /// Postgres in a container of this stack, with its data in the `pgdata` volume.
    #[default]
    Bundled,
    /// One that is already running somewhere.
    External(ExternalDb),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExternalDb {
    /// As the containers reach it: never localhost, which is the container itself.
    pub url: String,
    /// The Docker network it is a container on, when it is one. The API joins it.
    pub network: Option<String>,
}

/// What stands between the internet and the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proxy {
    /// The bundled Caddy on ports 80 and 443, with its own certificate.
    Caddy(Site),
    /// The bundled Caddy as a plain-http path router behind the operator's own proxy, which
    /// sends the whole site to it.
    Router(Router),
    /// No Caddy: the operator's proxy splits the paths and reaches the API and the frontend
    /// itself.
    Direct(Direct),
}

impl Default for Proxy {
    fn default() -> Self {
        Proxy::Caddy(Site::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    /// Empty means no domain: plain http on `http_port`.
    pub domain: String,
    pub http_port: u16,
    pub https_port: u16,
    /// Without a domain, where people open it, since only they know whether that is this
    /// machine's address, a tunnel or localhost. Empty for a domain, which says it already.
    pub public_url: String,
    /// Where the certificate for the domain comes from. Nothing without one.
    pub tls: Tls,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            domain: String::new(),
            http_port: 80,
            https_port: 443,
            public_url: String::new(),
            tls: Tls::Acme,
        }
    }
}

/// How the bundled Caddy gets a certificate for its domain. Each is a snippet in the
/// Caddyfile, picked by WS_TLS.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tls {
    /// Let's Encrypt, which needs the name to resolve here and ports 80 and 443 open.
    #[default]
    Acme,
    /// The operator's own, from `certs/`.
    Files,
    /// Caddy's own CA, for a private network where browsers can be told to trust it.
    Internal,
    /// None: plain http, for something in front that does TLS.
    Off,
}

impl Tls {
    pub const ALL: [Tls; 4] = [Tls::Acme, Tls::Files, Tls::Internal, Tls::Off];

    pub fn as_str(self) -> &'static str {
        match self {
            Tls::Acme => "acme",
            Tls::Files => "files",
            Tls::Internal => "internal",
            Tls::Off => "off",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tls| tls.as_str() == value)
    }
}

/// Where WS_TLS=files reads the certificate and key, as the checkout and the Caddy
/// container both see them.
pub const CERTS_DIR: &str = "certs";
pub const DEFAULT_CERT: &str = "fullchain.pem";
pub const DEFAULT_KEY: &str = "privkey.pem";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Router {
    /// What the browser uses, scheme included. The proxy in front decides it.
    pub public_url: String,
    /// The Docker network Caddy joins as `wormholesystems-router`.
    pub network: Option<String>,
    /// A loopback port Caddy is published on, for a proxy outside Docker.
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Direct {
    /// What the browser uses, scheme included. The proxy in front decides it.
    pub public_url: String,
    /// The Docker network the API and the frontend join, where a proxy container finds
    /// them as `wormholesystems-api` and `wormholesystems-web`.
    pub network: Option<String>,
    /// Loopback ports (API, frontend) for a proxy running on the machine itself rather than
    /// in Docker.
    pub host_ports: Option<(u16, u16)>,
}

pub const DEFAULT_NETWORK: &str = "wormholesystems";
pub const DEFAULT_ROUTER_PORT: u16 = 8080;

/// The compose files, base first. docker-compose.yml alone is the bundled everything, and
/// compose's default, so an install that is all bundled needs no COMPOSE_FILE at all.
pub const BASE_COMPOSE: &str = "docker-compose.yml";
pub const EXTERNAL_DB_COMPOSE: &str = "deploy/external-db.yml";
pub const DB_NETWORK_COMPOSE: &str = "deploy/db-network.yml";
pub const APP_NETWORK_COMPOSE: &str = "deploy/app-network.yml";
pub const APP_PORTS_COMPOSE: &str = "deploy/app-ports.yml";
pub const ROUTER_NETWORK_COMPOSE: &str = "deploy/router-network.yml";
pub const ROUTER_PORT_COMPOSE: &str = "deploy/router-port.yml";
/// What wsctl 0.5 wrote for a standalone install. Still in the checkout, so an install that
/// names them keeps running until `update` moves it to the overlays above.
pub const LEGACY_STANDALONE_COMPOSE: &str = "compose.standalone.yml";
pub const LEGACY_PORTS_COMPOSE: &str = "compose.standalone.ports.yml";

/// The files compose reads on its own next to docker-compose.yml, for changes of the
/// operator's that survive updates. Compose stops looking for them once COMPOSE_FILE is
/// set, so wsctl names whichever exist there itself.
pub const OVERRIDE_COMPOSE: [&str; 2] = ["compose.override.yml", "docker-compose.override.yml"];

/// Every compose file wsctl decides about. Anything else in COMPOSE_FILE was put there by
/// the operator, and stays.
const MANAGED_COMPOSE: [&str; 11] = [
    OVERRIDE_COMPOSE[0],
    OVERRIDE_COMPOSE[1],
    BASE_COMPOSE,
    EXTERNAL_DB_COMPOSE,
    DB_NETWORK_COMPOSE,
    APP_NETWORK_COMPOSE,
    APP_PORTS_COMPOSE,
    ROUTER_NETWORK_COMPOSE,
    ROUTER_PORT_COMPOSE,
    LEGACY_STANDALONE_COMPOSE,
    LEGACY_PORTS_COMPOSE,
];

impl Deployment {
    /// Read back what an earlier `setup` wrote. An install from before the two choices has
    /// WS_SETUP instead: `standalone` was an external database behind a proxy on the
    /// network (and the loopback ports when its COMPOSE_FILE had them), anything else all
    /// bundled.
    pub fn from_env(env: &str) -> Self {
        let get = |key: &str| envfile::get(env, key).unwrap_or_default();
        let filled = |key: &str| Some(get(key)).filter(|v| !v.is_empty());
        let port = |key: &str, fallback: u16| get(key).parse().unwrap_or(fallback);
        let legacy_standalone = get("WS_SETUP") == "standalone";

        let database = match get("WS_DATABASE").as_str() {
            "external" => Database::External(ExternalDb {
                url: get("WS_DATABASE_URL"),
                network: filled("WS_DATABASE_NETWORK"),
            }),
            "" if legacy_standalone => Database::External(ExternalDb {
                url: get("WS_DATABASE_URL"),
                network: None,
            }),
            _ => Database::Bundled,
        };

        let host_ports = || {
            filled("WS_API_PORT")
                .is_some()
                .then(|| (port("WS_API_PORT", 3000), port("WS_WEB_PORT", 3001)))
        };
        let proxy = match get("WS_PROXY").as_str() {
            "router" => {
                let network = filled("WS_PROXY_NETWORK");
                // Neither would leave Caddy on 80 and 443 with nothing in front of it; a
                // loopback port is the one that cannot surprise anybody.
                let port = (filled("WS_ROUTER_PORT").is_some() || network.is_none())
                    .then(|| port("WS_ROUTER_PORT", DEFAULT_ROUTER_PORT));
                Proxy::Router(Router {
                    public_url: get("WS_PUBLIC_URL"),
                    network,
                    port,
                })
            }
            "network" => Proxy::Direct(Direct {
                public_url: get("WS_PUBLIC_URL"),
                network: Some(filled("WS_PROXY_NETWORK").unwrap_or_else(|| DEFAULT_NETWORK.into())),
                host_ports: host_ports(),
            }),
            "ports" => Proxy::Direct(Direct {
                public_url: get("WS_PUBLIC_URL"),
                network: None,
                host_ports: Some(host_ports().unwrap_or((3000, 3001))),
            }),
            "" if legacy_standalone => Proxy::Direct(Direct {
                public_url: get("WS_PUBLIC_URL"),
                network: Some(filled("WS_PROXY_NETWORK").unwrap_or_else(|| DEFAULT_NETWORK.into())),
                host_ports: get("COMPOSE_FILE")
                    .contains(LEGACY_PORTS_COMPOSE)
                    .then(|| (port("WS_API_PORT", 3000), port("WS_WEB_PORT", 3001))),
            }),
            _ => {
                let domain = get("WS_DOMAIN");
                // As compose reads them, whatever they are: `setup` is what pins them to 80
                // and 443 for a domain, and an update must not quietly move somebody's ports.
                let (http_port, https_port) = (port("HTTP_PORT", 80), port("HTTPS_PORT", 443));
                Proxy::Caddy(Site {
                    public_url: if domain.is_empty() {
                        get("WS_PUBLIC_URL")
                    } else {
                        String::new()
                    },
                    domain,
                    http_port,
                    https_port,
                    tls: Tls::parse(&get("WS_TLS")).unwrap_or_default(),
                })
            }
        };
        Self { database, proxy }
    }

    /// Whether `.env` still says it the way wsctl 0.5 did, with WS_SETUP instead of the two
    /// choices.
    pub fn is_legacy(env: &str) -> bool {
        envfile::get(env, "WS_PROXY").is_none_or(|v| v.is_empty())
            && envfile::get(env, "WS_SETUP").is_some()
    }

    /// The compose files this needs, base first, then the database's overlays, then the
    /// proxy's. The order matters where two touch the same thing: router-port.yml replaces
    /// the ports router-network.yml cleared.
    pub fn compose_files(&self) -> Vec<&'static str> {
        let mut files = vec![BASE_COMPOSE];
        if let Database::External(db) = &self.database {
            files.push(EXTERNAL_DB_COMPOSE);
            // The API is on the proxy's network already; joining it twice under two names
            // is an error.
            if db.network.is_some() && db.network.as_deref() != self.app_network() {
                files.push(DB_NETWORK_COMPOSE);
            }
        }
        match &self.proxy {
            Proxy::Caddy(_) => {}
            Proxy::Router(router) => {
                if router.network.is_some() {
                    files.push(ROUTER_NETWORK_COMPOSE);
                }
                if router.port.is_some() {
                    files.push(ROUTER_PORT_COMPOSE);
                }
            }
            Proxy::Direct(direct) => {
                if direct.network.is_some() {
                    files.push(APP_NETWORK_COMPOSE);
                }
                if direct.host_ports.is_some() {
                    files.push(APP_PORTS_COMPOSE);
                }
            }
        }
        files
    }

    /// COMPOSE_FILE as it should read: wsctl's own files, then whatever the operator added
    /// to the current one, then the override files that exist (`overrides`), last so they
    /// win. None when compose's default reads exactly that already.
    pub fn compose_file(&self, current: &str, overrides: &[&str]) -> Option<String> {
        let mut files: Vec<&str> = self.compose_files();
        files.extend(
            current
                .split(':')
                .map(str::trim)
                .filter(|f| !f.is_empty() && !MANAGED_COMPOSE.contains(f)),
        );
        // docker-compose.yml alone picks up its overrides by itself.
        if files.len() == 1 {
            return None;
        }
        files.extend(overrides);
        Some(files.join(":"))
    }

    /// The network the API and the frontend are on for the operator's proxy, if any.
    pub fn app_network(&self) -> Option<&str> {
        match &self.proxy {
            Proxy::Direct(direct) => direct.network.as_deref(),
            _ => None,
        }
    }

    /// The network wsctl makes when it is missing: the one the operator's proxy joins.
    /// A database's network is somebody else's and has to exist already.
    pub fn proxy_network(&self) -> Option<&str> {
        match &self.proxy {
            Proxy::Router(router) => router.network.as_deref(),
            Proxy::Direct(direct) => direct.network.as_deref(),
            Proxy::Caddy(_) => None,
        }
    }

    /// What the outside world calls this install. Every redirect has to match it exactly.
    pub fn base_url(&self) -> String {
        match &self.proxy {
            Proxy::Caddy(site) if site.domain.is_empty() => {
                if site.public_url.is_empty() {
                    localhost(site.http_port)
                } else {
                    site.public_url.trim_end_matches('/').to_string()
                }
            }
            Proxy::Caddy(site) => {
                let (scheme, port, standard) = match site.tls {
                    Tls::Off => ("http", site.http_port, 80),
                    _ => ("https", site.https_port, 443),
                };
                if port == standard {
                    format!("{scheme}://{}", site.domain)
                } else {
                    format!("{scheme}://{}:{port}", site.domain)
                }
            }
            Proxy::Router(Router { public_url, .. }) | Proxy::Direct(Direct { public_url, .. }) => {
                public_url.trim_end_matches('/').to_string()
            }
        }
    }

    /// Where `status` checks that the install answers, when it has an address worth
    /// checking from here.
    pub fn public_url(&self) -> Option<String> {
        match &self.proxy {
            Proxy::Caddy(site) if site.domain.is_empty() && site.public_url.is_empty() => None,
            _ => Some(self.base_url()).filter(|u| !u.is_empty()),
        }
    }

    /// Write this into `.env`: its keys set, the keys of choices not taken struck out so a
    /// stale one cannot keep selecting them, and COMPOSE_FILE to match.
    ///
    /// `overrides` are the override files present in the checkout ([`overrides`]).
    pub fn apply(&self, env: &str, overrides: &[&str]) -> String {
        let mut values = BTreeMap::new();
        let mut removals = vec!["WS_SETUP"];
        let mut set = |k: &str, v: String| {
            values.insert(k.to_string(), v);
        };

        match &self.database {
            Database::Bundled => {
                set("WS_DATABASE", "bundled".into());
                removals.extend(["WS_DATABASE_URL", "WS_DATABASE_NETWORK"]);
            }
            Database::External(db) => {
                set("WS_DATABASE", "external".into());
                set("WS_DATABASE_URL", db.url.clone());
                match &db.network {
                    Some(network) => set("WS_DATABASE_NETWORK", network.clone()),
                    None => removals.push("WS_DATABASE_NETWORK"),
                }
            }
        }

        let (network, router_port, host_ports) = match &self.proxy {
            // WS_DOMAIN and the Caddy ports stay as they are for the other choices: nothing
            // reads them there, and going back to Caddy offers them as the defaults again.
            Proxy::Caddy(site) => {
                set("WS_PROXY", "caddy".into());
                set("WS_DOMAIN", site.domain.clone());
                set("HTTP_PORT", site.http_port.to_string());
                set("HTTPS_PORT", site.https_port.to_string());
                set("WS_TLS", site.tls.as_str().into());
                if site.domain.is_empty() && !site.public_url.is_empty() {
                    set("WS_PUBLIC_URL", self.base_url());
                } else {
                    removals.push("WS_PUBLIC_URL");
                }
                (None, None, None)
            }
            Proxy::Router(router) => {
                set("WS_PROXY", "router".into());
                set("WS_PUBLIC_URL", self.base_url());
                (router.network.as_ref(), router.port, None)
            }
            Proxy::Direct(direct) => {
                let kind = if direct.network.is_some() {
                    "network"
                } else {
                    "ports"
                };
                set("WS_PROXY", kind.into());
                set("WS_PUBLIC_URL", self.base_url());
                (direct.network.as_ref(), None, direct.host_ports)
            }
        };
        match network {
            Some(network) => set("WS_PROXY_NETWORK", network.clone()),
            None => removals.push("WS_PROXY_NETWORK"),
        }
        match router_port {
            Some(port) => set("WS_ROUTER_PORT", port.to_string()),
            None => removals.push("WS_ROUTER_PORT"),
        }
        match host_ports {
            Some((api, web)) => {
                set("WS_API_PORT", api.to_string());
                set("WS_WEB_PORT", web.to_string());
            }
            None => removals.extend(["WS_API_PORT", "WS_WEB_PORT"]),
        }

        let current = envfile::get(env, "COMPOSE_FILE").unwrap_or_default();
        match self.compose_file(&current, overrides) {
            Some(files) => set("COMPOSE_FILE", files),
            None => removals.push("COMPOSE_FILE"),
        }

        let mut out = envfile::patch(env, &values);
        for key in removals {
            out = envfile::remove(&out, key);
        }
        out
    }
}

/// The override files the checkout has, in the order compose would read them.
pub fn overrides(dir: &std::path::Path) -> Vec<&'static str> {
    OVERRIDE_COMPOSE
        .into_iter()
        .filter(|file| dir.join(file).is_file())
        .collect()
}

/// The address of a Caddy without a domain, seen from this machine.
pub fn localhost(port: u16) -> String {
    if port == 80 {
        "http://localhost".into()
    } else {
        format!("http://localhost:{port}")
    }
}

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
    pub fn base_url(&self) -> String {
        self.deployment.base_url()
    }

    /// The `.env` the answers make out of the current one.
    pub fn apply(&self, env: &str, overrides: &[&str]) -> String {
        let mut out = envfile::patch(&self.deployment.apply(env, overrides), &self.env_values());
        for key in self.discord_removals() {
            out = envfile::remove(&out, key);
        }
        out
    }

    /// The `.env` keys this produces besides the deployment's. Discord is absent rather
    /// than blank when it is off: the server reads it all-or-nothing, and an empty client
    /// id counts as configured.
    pub fn env_values(&self) -> BTreeMap<String, String> {
        let mut values = BTreeMap::new();
        let mut set = |k: &str, v: String| {
            values.insert(k.to_string(), v);
        };
        // Pinned rather than left to the checkout's directory name, which is what containers,
        // the network and the volumes are all named after.
        set("COMPOSE_PROJECT_NAME", crate::rename::PROJECT.into());
        set("WS_IMAGES", self.images.as_str().into());
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

    fn site(domain: &str) -> Proxy {
        Proxy::Caddy(Site {
            domain: domain.into(),
            ..Default::default()
        })
    }

    fn answers() -> Answers {
        Answers {
            deployment: Deployment {
                database: Database::Bundled,
                proxy: site("map.example.com"),
            },
            images: Images::Pull,
            contact_name: "Nicolas Kion".into(),
            contact_email: "me@example.com".into(),
            eve_client_id: "abc".into(),
            eve_client_secret: "shh".into(),
            discord: None,
            allowed_logins: String::new(),
            map_creators: String::new(),
        }
    }

    fn external(network: Option<&str>) -> Database {
        Database::External(ExternalDb {
            url: "postgres://ws:secret@postgres:5432/ws".into(),
            network: network.map(Into::into),
        })
    }

    fn direct(network: Option<&str>, host_ports: Option<(u16, u16)>) -> Proxy {
        Proxy::Direct(Direct {
            public_url: "https://map.example.com/".into(),
            network: network.map(Into::into),
            host_ports,
        })
    }

    fn router(network: Option<&str>, port: Option<u16>) -> Proxy {
        Proxy::Router(Router {
            public_url: "https://map.example.com".into(),
            network: network.map(Into::into),
            port,
        })
    }

    fn deployment(database: Database, proxy: Proxy) -> Deployment {
        Deployment { database, proxy }
    }

    fn get(env: &str, key: &str) -> Option<String> {
        envfile::get(env, key)
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
        let local = deployment(
            Database::Bundled,
            Proxy::Caddy(Site {
                http_port: 8080,
                ..Default::default()
            }),
        );
        assert_eq!(local.base_url(), "http://localhost:8080");
        assert_eq!(local.public_url(), None);
    }

    // Each certificate source reads back, and the URL follows the scheme it serves.
    #[test]
    fn the_certificate_source_decides_the_scheme() {
        for tls in Tls::ALL {
            let d = deployment(
                Database::Bundled,
                Proxy::Caddy(Site {
                    domain: "map.example.com".into(),
                    tls,
                    ..Default::default()
                }),
            );
            let env = d.apply("", &[]);
            assert_eq!(get(&env, "WS_TLS").as_deref(), Some(tls.as_str()));
            assert_eq!(Deployment::from_env(&env), d);
        }
        let plain = Deployment::from_env("WS_DOMAIN=map.example.com\nWS_TLS=off\n");
        assert_eq!(plain.base_url(), "http://map.example.com");
        let own_port =
            Deployment::from_env("WS_DOMAIN=map.example.com\nWS_TLS=internal\nHTTPS_PORT=8443\n");
        assert_eq!(own_port.base_url(), "https://map.example.com:8443");
        // Unset is what compose falls back to.
        let unset = Deployment::from_env("WS_DOMAIN=map.example.com\n");
        assert_eq!(unset.base_url(), "https://map.example.com");
    }

    // A server reached over SSH is not localhost to the browser that logs in to it.
    #[test]
    fn no_domain_takes_the_public_url_it_was_given() {
        let remote = deployment(
            Database::Bundled,
            Proxy::Caddy(Site {
                http_port: 8080,
                public_url: "http://203.0.113.5:8080/".into(),
                ..Default::default()
            }),
        );
        assert_eq!(remote.base_url(), "http://203.0.113.5:8080");
        let env = remote.apply("", &[]);
        assert_eq!(
            get(&env, "WS_PUBLIC_URL").as_deref(),
            Some("http://203.0.113.5:8080")
        );
        assert_eq!(
            Deployment::from_env(&env).public_url().as_deref(),
            Some("http://203.0.113.5:8080")
        );
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
        let env = answers().apply("DISCORD_CLIENT_ID=old\n", &[]);
        assert!(!env.contains("DISCORD_"));
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

    #[test]
    fn all_bundled_needs_no_compose_file() {
        let all = deployment(Database::Bundled, site(""));
        assert_eq!(all.compose_file("", &[]), None);
        let env = all.apply(
            "COMPOSE_FILE=compose.standalone.yml\nWS_SETUP=standalone\n",
            &[],
        );
        assert_eq!(get(&env, "COMPOSE_FILE"), None);
        assert_eq!(get(&env, "WS_SETUP"), None);
        assert_eq!(get(&env, "WS_DATABASE").as_deref(), Some("bundled"));
        assert_eq!(get(&env, "WS_PROXY").as_deref(), Some("caddy"));
    }

    // Each choice is one overlay, so every combination is the sum of its parts.
    #[test]
    fn each_choice_adds_its_overlay() {
        let cases = [
            (
                deployment(external(None), site("map.example.com")),
                "docker-compose.yml:deploy/external-db.yml",
            ),
            (
                deployment(Database::Bundled, direct(Some("web"), None)),
                "docker-compose.yml:deploy/app-network.yml",
            ),
            (
                deployment(Database::Bundled, direct(None, Some((3000, 3001)))),
                "docker-compose.yml:deploy/app-ports.yml",
            ),
            (
                deployment(external(Some("db")), router(Some("web"), None)),
                "docker-compose.yml:deploy/external-db.yml:deploy/db-network.yml:deploy/router-network.yml",
            ),
            (
                deployment(Database::Bundled, router(Some("web"), Some(8080))),
                "docker-compose.yml:deploy/router-network.yml:deploy/router-port.yml",
            ),
            (
                deployment(Database::Bundled, router(None, Some(8080))),
                "docker-compose.yml:deploy/router-port.yml",
            ),
        ];
        for (deployment, expected) in cases {
            assert_eq!(
                deployment.compose_file("", &[]).as_deref(),
                Some(expected),
                "{deployment:?}"
            );
        }
    }

    // The API is on the proxy's network already, and attaching it twice is an error.
    #[test]
    fn a_database_on_the_proxy_network_adds_nothing() {
        let shared = deployment(external(Some("coolify")), direct(Some("coolify"), None));
        assert_eq!(
            shared.compose_files(),
            vec![BASE_COMPOSE, EXTERNAL_DB_COMPOSE, APP_NETWORK_COMPOSE]
        );
    }

    // Compose reads compose.override.yml by itself only while COMPOSE_FILE is unset; with
    // one, an override that is not named in it is silently ignored.
    #[test]
    fn an_override_file_is_named_once_compose_file_is_needed() {
        let all = deployment(Database::Bundled, site(""));
        assert_eq!(all.compose_file("", &["compose.override.yml"]), None);
        let d = deployment(external(None), site(""));
        assert_eq!(
            d.compose_file(
                "docker-compose.yml:deploy/external-db.yml:compose.override.yml",
                &[]
            )
            .as_deref(),
            Some("docker-compose.yml:deploy/external-db.yml"),
            "a removed override goes from the list"
        );
        assert_eq!(
            d.compose_file("mine.yml", &["compose.override.yml"])
                .as_deref(),
            Some("docker-compose.yml:deploy/external-db.yml:mine.yml:compose.override.yml")
        );
    }

    #[test]
    fn files_the_operator_added_are_kept_after_ours() {
        let d = deployment(external(None), site(""));
        assert_eq!(
            d.compose_file(
                "compose.standalone.yml:mine.yml:compose.standalone.ports.yml",
                &[]
            )
            .as_deref(),
            Some("docker-compose.yml:deploy/external-db.yml:mine.yml")
        );
        let all = deployment(Database::Bundled, site(""));
        assert_eq!(
            all.compose_file("docker-compose.yml:mine.yml", &[])
                .as_deref(),
            Some("docker-compose.yml:mine.yml")
        );
    }

    // Behind somebody else's proxy the scheme is theirs to decide, so the URL is taken as
    // given (less a trailing slash) and every callback follows it.
    #[test]
    fn behind_a_proxy_the_public_url_is_taken_as_given() {
        let a = Answers {
            deployment: deployment(external(None), direct(Some("web"), None)),
            ..answers()
        };
        let env = a.apply("", &[]);
        assert_eq!(
            get(&env, "WS_PUBLIC_URL").as_deref(),
            Some("https://map.example.com")
        );
        assert_eq!(
            get(&env, "EVE_REDIRECT_URI").as_deref(),
            Some("https://map.example.com/auth/callback")
        );
    }

    #[test]
    fn the_keys_of_choices_not_taken_are_struck_out() {
        let before = "WS_DATABASE_URL=postgres://x@y/z\nWS_PROXY_NETWORK=web\nWS_API_PORT=1\nWS_WEB_PORT=2\nWS_ROUTER_PORT=3\nWS_PUBLIC_URL=https://x\nWS_DOMAIN=old.example.com\n";
        let env = deployment(Database::Bundled, site("map.example.com")).apply(before, &[]);
        for key in [
            "WS_DATABASE_URL",
            "WS_PROXY_NETWORK",
            "WS_API_PORT",
            "WS_WEB_PORT",
            "WS_ROUTER_PORT",
            "WS_PUBLIC_URL",
        ] {
            assert_eq!(get(&env, key), None, "{key} stays behind");
        }
        assert_eq!(get(&env, "WS_DOMAIN").as_deref(), Some("map.example.com"));

        // Leaving Caddy leaves its settings alone, to be offered again on the way back.
        let env = deployment(Database::Bundled, router(None, Some(8080))).apply(&env, &[]);
        assert_eq!(get(&env, "WS_DOMAIN").as_deref(), Some("map.example.com"));
        assert_eq!(get(&env, "WS_ROUTER_PORT").as_deref(), Some("8080"));
    }

    #[test]
    fn every_choice_reads_back_as_written() {
        let all = [
            deployment(Database::Bundled, site("map.example.com")),
            deployment(Database::Bundled, site("")),
            deployment(external(None), direct(Some("web"), None)),
            deployment(
                external(Some("db")),
                direct(Some("web"), Some((4000, 4001))),
            ),
            deployment(Database::Bundled, direct(None, Some((4000, 4001)))),
            deployment(external(Some("db")), router(Some("web"), None)),
            deployment(Database::Bundled, router(None, Some(9000))),
            deployment(Database::Bundled, router(Some("web"), Some(9000))),
        ];
        for written in all {
            let env = written.apply("", &[]);
            let mut expected = written.clone();
            if let Proxy::Direct(d) = &mut expected.proxy {
                d.public_url = "https://map.example.com".into();
            }
            assert_eq!(Deployment::from_env(&env), expected, "{env}");
        }
    }

    // What wsctl 0.5 wrote has to keep meaning the same stack, so an update can rewrite it.
    #[test]
    fn a_standalone_install_from_before_reads_as_its_two_choices() {
        let env = "WS_SETUP=standalone\nCOMPOSE_FILE=compose.standalone.yml:compose.standalone.ports.yml\nWS_PUBLIC_URL=https://map.example.com\nWS_DATABASE_URL=postgres://ws:secret@postgres:5432/ws\nWS_PROXY_NETWORK=coolify\nWS_API_PORT=4000\nWS_WEB_PORT=4001\n";
        assert!(Deployment::is_legacy(env));
        let read = Deployment::from_env(env);
        assert_eq!(
            read,
            deployment(
                external(None),
                Proxy::Direct(Direct {
                    public_url: "https://map.example.com".into(),
                    network: Some("coolify".into()),
                    host_ports: Some((4000, 4001)),
                })
            )
        );
        let rewritten = read.apply(env, &[]);
        assert_eq!(
            get(&rewritten, "COMPOSE_FILE").as_deref(),
            Some(
                "docker-compose.yml:deploy/external-db.yml:deploy/app-network.yml:deploy/app-ports.yml"
            )
        );
        assert_eq!(get(&rewritten, "WS_PROXY").as_deref(), Some("network"));
        assert_eq!(get(&rewritten, "WS_DATABASE").as_deref(), Some("external"));
        assert!(!Deployment::is_legacy(&rewritten));
        assert_eq!(Deployment::from_env(&rewritten), read);
    }

    #[test]
    fn a_full_install_from_before_is_all_bundled() {
        let env = "WS_SETUP=full\nWS_DOMAIN=map.example.com\n";
        assert_eq!(
            Deployment::from_env(env),
            deployment(Database::Bundled, site("map.example.com"))
        );
        // And one from before even WS_SETUP.
        assert_eq!(
            Deployment::from_env("WS_DOMAIN=map.example.com\n"),
            deployment(Database::Bundled, site("map.example.com"))
        );
    }

    #[test]
    fn the_image_source_is_written_and_read_back() {
        let building = Answers {
            images: Images::Build,
            ..answers()
        };
        let env = envfile::patch("", &building.env_values());
        assert_eq!(Images::from_env(&env), Images::Build);
        let env = envfile::patch("", &answers().env_values());
        assert_eq!(Images::from_env(&env), Images::Pull);
    }

    // Pulling is the default for a new install, not for one that has always built: its
    // operator never agreed to run somebody else's image.
    #[test]
    fn an_install_that_never_chose_keeps_building() {
        assert_eq!(
            Images::from_env("WS_DOMAIN=map.example.com\n"),
            Images::Build
        );
        assert!(!Images::chosen("WS_DOMAIN=map.example.com\n"));
    }

    #[test]
    fn the_generated_password_needs_no_escaping() {
        let password = generated_password();
        assert_eq!(password.len(), 32);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
