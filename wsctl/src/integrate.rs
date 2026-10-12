//! What a reverse proxy in front of a standalone install needs to be told.
//!
//! The bundled Caddyfile is the reference: the same paths go to the API, everything else to
//! the frontend, and WebSocket upgrades have to pass through. A test holds the list here to
//! that file, so the two cannot drift apart.

use console::style;

use crate::config::Standalone;
use crate::ui;

/// The paths the API answers. Everything else is the frontend's.
pub const API_PATHS: [&str; 4] = ["/api/*", "/ws/*", "/auth/*", "/discord/*"];

pub const API_HOST: &str = "wormholesystems-api:3000";
pub const WEB_HOST: &str = "wormholesystems-web:3001";

/// The largest body the API accepts (a map import, 32MB), plus room for the multipart
/// framing. nginx refuses anything over 1MB unless told otherwise.
const MAX_BODY: &str = "33m";

/// A site block for a Caddy that shares the Docker network with the app.
pub fn caddy(standalone: &Standalone) -> String {
    format!(
        "{site} {{\n\t@api path {paths}\n\thandle @api {{\n\t\treverse_proxy {API_HOST}\n\t}}\n\thandle {{\n\t\treverse_proxy {WEB_HOST}\n\t}}\n}}\n",
        site = standalone.public_url.trim_end_matches('/'),
        paths = API_PATHS.join(" "),
    )
}

/// A server block for an nginx on this machine, reaching the app on its loopback ports.
pub fn nginx(standalone: &Standalone, api_port: u16, web_port: u16) -> String {
    let host = host_of(&standalone.public_url);
    let prefixes = API_PATHS
        .iter()
        .map(|p| p.trim_start_matches('/').trim_end_matches("/*"))
        .collect::<Vec<_>>()
        .join("|");
    format!(
        "server {{
    server_name {host};
    # listen and TLS as for the other sites on this machine
    client_max_body_size {MAX_BODY};

    location ~ ^/({prefixes})/ {{
        proxy_pass http://127.0.0.1:{api_port};
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection \"upgrade\";
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        # The map's live updates hold a WebSocket open for as long as the tab is.
        proxy_read_timeout 1h;
    }}

    location / {{
        proxy_pass http://127.0.0.1:{web_port};
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}
}}
"
    )
}

/// The hostname out of a URL, for the places that want no scheme or path.
fn host_of(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    rest.split('/').next().unwrap_or(rest)
}

pub fn print(standalone: &Standalone) {
    ui::heading("Pointing your proxy at it");
    ui::note(&format!(
        "Send {} to the API, with WebSocket upgrades passed through,",
        API_PATHS.join(", ")
    ));
    ui::note("and everything else to the frontend:");
    let rows = std::iter::once((
        format!("on the Docker network {}:", standalone.network),
        format!("{API_HOST} and {WEB_HOST}"),
    ))
    .chain(standalone.host_ports.map(|(api, web)| {
        (
            "on this machine:".to_string(),
            format!("127.0.0.1:{api} and 127.0.0.1:{web}"),
        )
    }))
    .collect::<Vec<_>>();
    let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    for (label, target) in &rows {
        ui::note(&format!("  {label:<width$}  {target}"));
    }
    ui::note(&format!(
        "The public URL must be {}; the EVE and Discord callbacks are built from it.",
        standalone.public_url
    ));

    println!(
        "\n  {}",
        style("Caddy in Docker, on the same network:").bold()
    );
    print_block(&caddy(standalone));
    if let Some((api, web)) = standalone.host_ports {
        println!("\n  {}", style("nginx on this machine:").bold());
        print_block(&nginx(standalone, api, web));
    }
}

fn print_block(text: &str) {
    for line in text.lines() {
        println!("    {}", style(line).cyan());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standalone() -> Standalone {
        Standalone {
            public_url: "https://map.example.com/".into(),
            database_url: String::new(),
            network: "web".into(),
            host_ports: Some((3000, 3001)),
        }
    }

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

    // A path the full stack sends to the API but a standalone proxy sends to the frontend
    // fails in exactly one setup, which is the kind of bug nobody finds until a user does.
    #[test]
    fn the_api_paths_match_the_bundled_caddyfile() {
        assert_eq!(bundled_api_paths(), API_PATHS);
    }

    #[test]
    fn the_caddy_block_routes_the_api_paths_and_the_rest() {
        assert_eq!(
            caddy(&standalone()),
            "https://map.example.com {\n\t@api path /api/* /ws/* /auth/* /discord/*\n\thandle @api {\n\t\treverse_proxy wormholesystems-api:3000\n\t}\n\thandle {\n\t\treverse_proxy wormholesystems-web:3001\n\t}\n}\n"
        );
    }

    #[test]
    fn the_nginx_block_uses_the_hostname_and_the_loopback_ports() {
        let block = nginx(&standalone(), 4000, 4001);
        assert!(block.contains("server_name map.example.com;"));
        assert!(block.contains("location ~ ^/(api|ws|auth|discord)/ {"));
        assert!(block.contains("proxy_pass http://127.0.0.1:4000;"));
        assert!(block.contains("proxy_pass http://127.0.0.1:4001;"));
        assert!(block.contains("proxy_set_header Upgrade $http_upgrade;"));
        assert!(block.contains("client_max_body_size 33m;"));
    }

    #[test]
    fn the_host_is_the_url_without_scheme_or_path() {
        assert_eq!(host_of("https://map.example.com/x"), "map.example.com");
        assert_eq!(host_of("http://localhost:8080"), "localhost:8080");
        assert_eq!(host_of("map.example.com"), "map.example.com");
    }
}
