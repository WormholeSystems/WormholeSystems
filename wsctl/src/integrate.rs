//! What a reverse proxy in front of the install needs to be told, for each way it can
//! stand there.
//!
//! The bundled Caddyfile is the reference: the same paths go to the API, everything else to
//! the frontend, and WebSocket upgrades have to pass through. A test holds the list here to
//! that file, so the two cannot drift apart.

use console::style;

use crate::config::{Deployment, Direct, Proxy, Router, Site};
use crate::ui;

/// The paths the API answers. Everything else is the frontend's.
pub const API_PATHS: [&str; 4] = ["/api/*", "/ws/*", "/auth/*", "/discord/*"];

pub const API_HOST: &str = "wormholesystems-api:3000";
pub const WEB_HOST: &str = "wormholesystems-web:3001";
/// The bundled Caddy as the path router, on the network it shares with the proxy.
pub const ROUTER_HOST: &str = "wormholesystems-router:80";

/// The largest body the API accepts (a map import, 32MB), plus room for the multipart
/// framing. nginx refuses anything over 1MB unless told otherwise.
const MAX_BODY: &str = "33m";

/// A site block for a Caddy that shares the Docker network with the API and the frontend.
pub fn caddy_split(public_url: &str) -> String {
    format!(
        "{site} {{\n\t@api path {paths}\n\thandle @api {{\n\t\treverse_proxy {API_HOST}\n\t}}\n\thandle {{\n\t\treverse_proxy {WEB_HOST}\n\t}}\n}}\n",
        site = public_url.trim_end_matches('/'),
        paths = API_PATHS.join(" "),
    )
}

/// A site block for a Caddy that sends the whole site to one upstream.
pub fn caddy_single(public_url: &str, upstream: &str) -> String {
    format!(
        "{site} {{\n\treverse_proxy {upstream}\n}}\n",
        site = public_url.trim_end_matches('/'),
    )
}

/// The headers every nginx location here sets: the original host, scheme and address, and
/// the upgrade a WebSocket needs.
const NGINX_HEADERS: &str = "        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;";

/// `Connection: upgrade` only for a request that asked for one, so a plain request keeps
/// its keep-alive. Goes in the http block, once.
const NGINX_MAP: &str = "map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}
";

/// A server block for an nginx on this machine, splitting the paths itself between the
/// API's and the frontend's loopback ports.
pub fn nginx_split(public_url: &str, api_port: u16, web_port: u16) -> String {
    let host = host_of(public_url);
    let prefixes = API_PATHS
        .iter()
        .map(|p| p.trim_start_matches('/').trim_end_matches("/*"))
        .collect::<Vec<_>>()
        .join("|");
    format!(
        "{NGINX_MAP}
server {{
    server_name {host};
    # listen and TLS as for the other sites on this machine
    client_max_body_size {MAX_BODY};

    location ~ ^/({prefixes})/ {{
        proxy_pass http://127.0.0.1:{api_port};
{NGINX_HEADERS}
        # The map's live updates hold a WebSocket open for as long as the tab is.
        proxy_read_timeout 1h;
    }}

    location / {{
        proxy_pass http://127.0.0.1:{web_port};
{NGINX_HEADERS}
    }}
}}
"
    )
}

/// A server block for an nginx on this machine that sends everything to the bundled
/// Caddy's loopback port.
pub fn nginx_single(public_url: &str, port: u16) -> String {
    let host = host_of(public_url);
    format!(
        "{NGINX_MAP}
server {{
    server_name {host};
    # listen and TLS as for the other sites on this machine
    client_max_body_size {MAX_BODY};

    location / {{
        proxy_pass http://127.0.0.1:{port};
{NGINX_HEADERS}
        # The map's live updates hold a WebSocket open for as long as the tab is.
        proxy_read_timeout 1h;
    }}
}}
"
    )
}

/// Labels for Traefik, which is what Coolify runs, as a compose.override.yml: the file
/// survives updates and wsctl adds it to COMPOSE_FILE. One router per service, the API's
/// with the path prefixes; Traefik tries the longer rule first, so it wins over the
/// frontend's catch-all.
pub fn traefik(public_url: &str, network: &str, services: &[(&str, &str, u16)]) -> String {
    let host = host_of(public_url);
    let mut out = String::from(
        "# compose.override.yml\n# Coolify's entrypoint is `https` and its resolver `letsencrypt`; a plain Traefik\n# usually calls them `websecure` and whatever its static config names.\nservices:\n",
    );
    for (service, rule, port) in services {
        let name = format!("wormholesystems-{service}");
        out.push_str(&format!(
            "  {service}:
    labels:
      - traefik.enable=true
      - traefik.docker.network={network}
      - traefik.http.routers.{name}.rule={rule}
      - traefik.http.routers.{name}.entrypoints=https
      - traefik.http.routers.{name}.tls=true
      - traefik.http.routers.{name}.tls.certresolver=letsencrypt
      - traefik.http.services.{name}.loadbalancer.server.port={port}
",
            rule = rule.replace("{host}", host),
        ));
    }
    out
}

/// The Traefik rule for the API's paths.
fn traefik_api_rule() -> String {
    let prefixes = API_PATHS
        .iter()
        .map(|p| format!("PathPrefix(`{}`)", p.trim_end_matches('*')))
        .collect::<Vec<_>>()
        .join(" || ");
    format!("Host(`{{host}}`) && ({prefixes})")
}

const TRAEFIK_HOST_RULE: &str = "Host(`{host}`)";

/// The hostname out of a URL, for the places that want no scheme or path.
fn host_of(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    rest.split('/').next().unwrap_or(rest)
}

pub fn print(deployment: &Deployment) {
    match &deployment.proxy {
        Proxy::Caddy(site) => print_caddy(site),
        Proxy::Router(router) => print_router(router),
        Proxy::Direct(direct) => print_direct(direct),
    }
}

fn print_caddy(site: &Site) {
    ui::heading("Nothing to point");
    ui::note("The bundled Caddy is the proxy here, on ports 80 and 443.");
    if site.domain.is_empty() {
        ui::note("Without a domain it serves plain http on this machine.");
    } else {
        ui::note(&format!(
            "Point {} at this machine; Caddy gets and renews the certificate.",
            site.domain
        ));
    }
    ui::note("To put your own proxy in front instead, run `wsctl setup` and pick that.");
}

fn print_router(router: &Router) {
    let url = &router.public_url;
    ui::heading("Pointing your proxy at it");
    ui::note("The bundled Caddy splits the paths, so your proxy sends the whole site, WebSocket");
    ui::note("upgrades included, to one place:");
    let mut rows = Vec::new();
    if let Some(network) = &router.network {
        rows.push((
            format!("on the Docker network {network}:"),
            ROUTER_HOST.to_string(),
        ));
    }
    if let Some(port) = router.port {
        rows.push(("on this machine:".to_string(), format!("127.0.0.1:{port}")));
    }
    print_rows(&rows);
    note_url(url);

    if router.network.is_some() {
        block(
            "Caddy in Docker, on the same network:",
            &caddy_single(url, ROUTER_HOST),
        );
    }
    if let Some(network) = &router.network {
        block(
            "Traefik or Coolify, on the same network:",
            &traefik(url, network, &[("caddy", TRAEFIK_HOST_RULE, 80)]),
        );
    }
    if let Some(port) = router.port {
        block("nginx on this machine:", &nginx_single(url, port));
    }
}

fn print_direct(direct: &Direct) {
    let url = &direct.public_url;
    ui::heading("Pointing your proxy at it");
    ui::note(&format!(
        "Send {} to the API, with WebSocket upgrades passed through,",
        API_PATHS.join(", ")
    ));
    ui::note("and everything else to the frontend:");
    let mut rows = Vec::new();
    if let Some(network) = &direct.network {
        rows.push((
            format!("on the Docker network {network}:"),
            format!("{API_HOST} and {WEB_HOST}"),
        ));
    }
    if let Some((api, web)) = direct.host_ports {
        rows.push((
            "on this machine:".to_string(),
            format!("127.0.0.1:{api} and 127.0.0.1:{web}"),
        ));
    }
    print_rows(&rows);
    note_url(url);

    if let Some(network) = &direct.network {
        block("Caddy in Docker, on the same network:", &caddy_split(url));
        let api_rule = traefik_api_rule();
        block(
            "Traefik or Coolify, on the same network:",
            &traefik(
                url,
                network,
                &[("api", &api_rule, 3000), ("web", TRAEFIK_HOST_RULE, 3001)],
            ),
        );
    }
    if let Some((api, web)) = direct.host_ports {
        block("nginx on this machine:", &nginx_split(url, api, web));
    }
}

fn print_rows(rows: &[(String, String)]) {
    let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    for (label, target) in rows {
        ui::note(&format!("  {label:<width$}  {target}"));
    }
}

fn note_url(url: &str) {
    ui::note(&format!(
        "The public URL must be {url}; the EVE and Discord callbacks are built from it."
    ));
}

fn block(title: &str, text: &str) {
    println!("\n  {}", style(title).bold());
    for line in text.lines() {
        println!("    {}", style(line).cyan());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://map.example.com/";

    /// The paths the bundled Caddyfile hands to the API, in its order.
    fn bundled_api_paths() -> Vec<String> {
        let caddyfile = include_str!("../../Caddyfile");
        let lines: Vec<&str> = caddyfile
            .lines()
            .map(str::trim)
            .filter(|l| !l.starts_with('#'))
            .collect();
        lines
            .windows(2)
            .filter(|pair| pair[1] == "reverse_proxy api:3000")
            .filter_map(|pair| pair[0].strip_prefix("handle ")?.strip_suffix(" {"))
            .map(str::to_string)
            .collect()
    }

    // A path the bundled Caddy sends to the API but your own proxy sends to the frontend
    // fails in exactly one setup, which is the kind of bug nobody finds until a user does.
    #[test]
    fn the_api_paths_match_the_bundled_caddyfile() {
        assert_eq!(bundled_api_paths(), API_PATHS);
    }

    #[test]
    fn the_caddy_block_routes_the_api_paths_and_the_rest() {
        assert_eq!(
            caddy_split(URL),
            "https://map.example.com {\n\t@api path /api/* /ws/* /auth/* /discord/*\n\thandle @api {\n\t\treverse_proxy wormholesystems-api:3000\n\t}\n\thandle {\n\t\treverse_proxy wormholesystems-web:3001\n\t}\n}\n"
        );
    }

    #[test]
    fn behind_the_router_one_upstream_takes_everything() {
        assert_eq!(
            caddy_single(URL, ROUTER_HOST),
            "https://map.example.com {\n\treverse_proxy wormholesystems-router:80\n}\n"
        );
        let block = nginx_single(URL, 8080);
        assert!(block.contains("server_name map.example.com;"));
        assert!(block.contains("proxy_pass http://127.0.0.1:8080;"));
        assert!(block.contains("proxy_set_header Upgrade $http_upgrade;"));
        assert!(!block.contains("location ~"));
    }

    #[test]
    fn the_nginx_block_uses_the_hostname_and_the_loopback_ports() {
        let block = nginx_split(URL, 4000, 4001);
        assert!(block.contains("server_name map.example.com;"));
        assert!(block.contains("location ~ ^/(api|ws|auth|discord)/ {"));
        assert!(block.contains("proxy_pass http://127.0.0.1:4000;"));
        assert!(block.contains("proxy_pass http://127.0.0.1:4001;"));
        assert!(block.contains("proxy_set_header Upgrade $http_upgrade;"));
        assert!(block.contains("client_max_body_size 33m;"));
    }

    #[test]
    fn the_traefik_labels_route_the_api_paths_before_the_rest() {
        let labels = traefik(
            URL,
            "coolify",
            &[
                ("api", &traefik_api_rule(), 3000),
                ("web", TRAEFIK_HOST_RULE, 3001),
            ],
        );
        assert!(labels.contains("  api:\n    labels:\n"));
        assert!(labels.contains(
            "traefik.http.routers.wormholesystems-api.rule=Host(`map.example.com`) && (PathPrefix(`/api/`) || PathPrefix(`/ws/`) || PathPrefix(`/auth/`) || PathPrefix(`/discord/`))"
        ));
        assert!(
            labels.contains(
                "traefik.http.routers.wormholesystems-web.rule=Host(`map.example.com`)\n"
            )
        );
        assert!(labels.contains("traefik.docker.network=coolify"));
        assert!(labels.contains("wormholesystems-web.loadbalancer.server.port=3001"));
    }

    #[test]
    fn the_host_is_the_url_without_scheme_or_path() {
        assert_eq!(host_of("https://map.example.com/x"), "map.example.com");
        assert_eq!(host_of("http://localhost:8080"), "localhost:8080");
        assert_eq!(host_of("map.example.com"), "map.example.com");
    }
}
