//! What the machine is asked about before anything is asked of the operator.
//!
//! Each check answers with a verdict rather than printing and exiting, so `doctor` can show
//! them all and `setup` can decide which ones are worth stopping for.

use std::net::{IpAddr, ToSocketAddrs};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Result;

use crate::config::Standalone;
use crate::exec::Runner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Ok,
    /// Worth saying, not worth stopping for.
    Warn,
    /// Nothing will work until this is dealt with.
    Fatal,
}

#[derive(Debug, Clone)]
pub struct Check {
    pub verdict: Verdict,
    pub label: String,
    /// What to do about it, when there is something to do.
    pub hint: Option<String>,
}

impl Check {
    pub fn ok(label: impl Into<String>) -> Self {
        Self {
            verdict: Verdict::Ok,
            label: label.into(),
            hint: None,
        }
    }
    pub fn warn(label: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            verdict: Verdict::Warn,
            label: label.into(),
            hint: Some(hint.into()),
        }
    }
    pub fn fatal(label: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            verdict: Verdict::Fatal,
            label: label.into(),
            hint: Some(hint.into()),
        }
    }
}

pub fn docker(runner: &mut dyn Runner, dir: &Path) -> Check {
    let Ok(version) = runner.capture(
        dir,
        "docker",
        &["version", "--format", "{{.Server.Version}}"],
    ) else {
        return Check::fatal(
            "Docker is not running, or this user cannot reach it",
            "Install it from https://docs.docker.com/engine/install/, then start it.",
        );
    };
    if runner
        .capture(dir, "docker", &["compose", "version"])
        .is_err()
    {
        return Check::fatal(
            "Docker Compose v2 is missing",
            "This needs the `docker compose` subcommand, not the old `docker-compose`.",
        );
    }
    Check::ok(format!("Docker {version} with Compose v2"))
}

/// The release build wants roughly 2GB of memory and the static data a few GB of disk.
pub fn disk(path: &Path) -> Check {
    let Some(free) = free_gb(path) else {
        return Check::warn("could not read the free disk space", "Check it by hand.");
    };
    if free < 5 {
        Check::warn(
            format!("{free}GB free here"),
            "A build plus the static data wants about 5GB.",
        )
    } else {
        Check::ok(format!("{free}GB free"))
    }
}

/// `ours` says this stack's own proxy is running, which is what holds these ports on a
/// machine that is already serving. Busy is only a problem when it is somebody else.
pub fn ports(ports: &[u16], ours: bool) -> Vec<Check> {
    ports
        .iter()
        .map(|&port| match (port_is_free(port), ours) {
            (true, _) => Check::ok(format!("port {port} is free")),
            (false, true) => Check::ok(format!("port {port} is held by this stack")),
            (false, false) => Check::fatal(
                format!("port {port} is already in use"),
                "Stop whatever is on it, or pick another port in `wsctl setup`.",
            ),
        })
        .collect()
}

/// Whether the name points here. Certificates cannot be issued for a name that does not
/// resolve, and Let's Encrypt has to reach port 80 on this machine.
pub fn dns(domain: &str, public: Option<IpAddr>) -> Check {
    if domain.is_empty() {
        return Check::ok("no domain, so plain http on this machine");
    }
    let resolved = resolve(domain);
    if resolved.is_empty() {
        return Check::fatal(
            format!("{domain} does not resolve"),
            "Point an A record at this machine, then run this again.",
        );
    }
    let list = resolved
        .iter()
        .map(|ip| ip.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    match public {
        Some(ip) if resolved.contains(&ip) => Check::ok(format!("{domain} resolves here ({ip})")),
        Some(ip) => Check::warn(
            format!("{domain} resolves to {list}, but this machine looks like {ip}"),
            "Fine behind Cloudflare or a NAT. Otherwise certificates will not be issued.",
        ),
        None => Check::warn(
            format!("{domain} resolves to {list}"),
            "Could not work out this machine's public address to compare.",
        ),
    }
}

/// Whether the checkout is behind its remote, so an update is not a surprise later.
pub fn repo(runner: &mut dyn Runner, dir: &Path) -> Check {
    if runner
        .capture(dir, "git", &["rev-parse", "--git-dir"])
        .is_err()
    {
        return Check::warn(
            "not a git checkout",
            "Updates will have to be done by hand.",
        );
    }
    if runner.run(dir, "git", &["fetch", "--quiet"]).is_err() {
        return Check::warn("could not reach the remote", "Skipping the version check.");
    }
    match runner.capture(dir, "git", &["rev-list", "--count", "HEAD..@{u}"]) {
        Ok(behind) if behind != "0" && !behind.is_empty() => Check::warn(
            format!("{behind} commit(s) behind the remote"),
            "`wsctl update` takes them.",
        ),
        _ => Check::ok("up to date with the remote"),
    }
}

/// Whether the database a standalone install points at answers where the API will look
/// for it: inside the Docker network, not on this machine's loopback.
pub fn database(runner: &mut dyn Runner, dir: &Path, network: &str, url: &str) -> Check {
    let Some((host, port)) = database_address(url) else {
        return Check::fatal(
            "WS_DATABASE_URL is not a postgres:// URL",
            "It looks like postgres://user:password@host:5432/database.",
        );
    };
    if matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1") {
        return Check::fatal(
            format!("the database is on {host}, which inside a container is the container"),
            "Use the database's name on the Docker network, or this machine's address.",
        );
    }
    if crate::stack::reachable_from(runner, dir, network, &host, port) {
        Check::ok(format!("Postgres at {host}:{port} answers from {network}"))
    } else {
        Check::warn(
            format!("nothing answers at {host}:{port} from the network {network}"),
            "Put the database on that network or use an address it can reach; the API will not start until it can connect.",
        )
    }
}

pub fn network(runner: &mut dyn Runner, dir: &Path, name: &str) -> Check {
    if crate::stack::network_exists(runner, dir, name) {
        Check::ok(format!("the Docker network {name} exists"))
    } else {
        Check::fatal(
            format!("there is no Docker network {name}"),
            "`wsctl setup` creates it, or make it with `docker network create`.",
        )
    }
}

/// Host and port out of a postgres URL, the port defaulting the way libpq's does.
pub fn database_address(url: &str) -> Option<(String, u16)> {
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))?;
    let authority = rest.split(['/', '?']).next()?;
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, hp)| hp);
    let (host, port) = match host_port.strip_prefix('[') {
        // [::1]:5432
        Some(v6) => {
            let (host, after) = v6.split_once(']')?;
            (host, after.strip_prefix(':'))
        }
        None => match host_port.rsplit_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (host_port, None),
        },
    };
    if host.is_empty() {
        return None;
    }
    let port = match port {
        Some(p) => p.parse().ok()?,
        None => 5432,
    };
    Some((host.to_string(), port))
}

pub fn worst(checks: &[Check]) -> Verdict {
    checks
        .iter()
        .map(|c| c.verdict)
        .max_by_key(|v| match v {
            Verdict::Ok => 0,
            Verdict::Warn => 1,
            Verdict::Fatal => 2,
        })
        .unwrap_or(Verdict::Ok)
}

fn resolve(domain: &str) -> Vec<IpAddr> {
    (domain, 0u16)
        .to_socket_addrs()
        .map(|addrs| addrs.map(|a| a.ip()).collect())
        .unwrap_or_default()
}

fn port_is_free(port: u16) -> bool {
    std::net::TcpListener::bind(("0.0.0.0", port)).is_ok()
}

fn free_gb(path: &Path) -> Option<u64> {
    let out = Command::new("df")
        .args(["-Pk", path.to_str()?])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let available: u64 = text
        .lines()
        .nth(1)?
        .split_whitespace()
        .nth(3)?
        .parse()
        .ok()?;
    Some(available / 1024 / 1024)
}

/// This machine's address as the internet sees it. Best effort: a warning either way.
pub fn public_ip() -> Option<IpAddr> {
    for url in ["https://api.ipify.org", "https://ifconfig.me"] {
        let out = Command::new("curl")
            .args(["-fsS", "--max-time", "5", url])
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if let Ok(ip) = String::from_utf8_lossy(&out.stdout).trim().parse() {
            return Some(ip);
        }
    }
    None
}

pub fn all(
    runner: &mut dyn Runner,
    dir: &Path,
    domain: &str,
    http: u16,
    https: u16,
) -> Result<Vec<Check>> {
    let ours = crate::stack::proxy_running(runner, dir);
    let mut checks = vec![docker(runner, dir), disk(dir), repo(runner, dir)];
    checks.extend(ports(&[http, https], ours));
    checks.push(dns(domain, public_ip()));
    Ok(checks)
}

/// What `doctor` looks at for a standalone install: no ports 80 and 443 and no DNS, those
/// belong to the proxy in front; the network, the database, and the loopback ports instead.
pub fn standalone(runner: &mut dyn Runner, dir: &Path, standalone: &Standalone) -> Vec<Check> {
    let mut checks = vec![docker(runner, dir), disk(dir), repo(runner, dir)];
    let network = network(runner, dir, &standalone.network);
    let network_ok = network.verdict == Verdict::Ok;
    checks.push(network);
    if network_ok {
        checks.push(database(
            runner,
            dir,
            &standalone.network,
            &standalone.database_url,
        ));
    }
    if let Some((api, web)) = standalone.host_ports {
        let ours = crate::stack::service_running(runner, dir, "api");
        checks.extend(ports(&[api, web], ours));
    }
    checks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_that_does_not_resolve_is_fatal() {
        let check = dns("nothing.invalid", None);
        assert_eq!(check.verdict, Verdict::Fatal);
    }

    #[test]
    fn no_domain_is_fine() {
        assert_eq!(dns("", None).verdict, Verdict::Ok);
    }

    #[test]
    fn a_name_pointing_elsewhere_warns_rather_than_stops() {
        // Behind Cloudflare this is the normal case, so it cannot be fatal.
        let check = dns("localhost", Some("203.0.113.1".parse().unwrap()));
        assert_eq!(check.verdict, Verdict::Warn);
    }

    #[test]
    fn a_port_something_else_is_listening_on_is_fatal() {
        let held = std::net::TcpListener::bind(("0.0.0.0", 0)).unwrap();
        let port = held.local_addr().unwrap().port();
        assert_eq!(ports(&[port], false)[0].verdict, Verdict::Fatal);
    }

    // An install that is already serving holds its own ports, and telling its operator to
    // stop whatever is on them means telling them to stop the thing they are updating.
    #[test]
    fn a_port_this_stack_holds_is_fine() {
        let held = std::net::TcpListener::bind(("0.0.0.0", 0)).unwrap();
        let port = held.local_addr().unwrap().port();
        assert_eq!(ports(&[port], true)[0].verdict, Verdict::Ok);
    }

    #[test]
    fn the_database_address_comes_out_of_the_url() {
        let cases = [
            (
                "postgres://ws:secret@postgres:5433/ws",
                Some(("postgres", 5433)),
            ),
            (
                "postgresql://ws@db.internal/ws?sslmode=disable",
                Some(("db.internal", 5432)),
            ),
            ("postgres://ws:p%40ss@10.0.0.5/ws", Some(("10.0.0.5", 5432))),
            ("postgres://ws:secret@[::1]:5432/ws", Some(("::1", 5432))),
            ("mysql://ws@db/ws", None),
            ("postgres:///ws", None),
        ];
        for (url, expected) in cases {
            assert_eq!(
                database_address(url),
                expected.map(|(h, p)| (h.to_string(), p)),
                "{url}"
            );
        }
    }

    // The API runs in a container, so the operator's localhost is not the API's localhost.
    // This is the most likely wrong answer, and docker would only say "connection refused".
    #[test]
    fn a_database_on_localhost_is_fatal() {
        let mut runner = crate::exec::Recording::default();
        let check = database(
            &mut runner,
            Path::new("."),
            "web",
            "postgres://ws:secret@localhost:5432/ws",
        );
        assert_eq!(check.verdict, Verdict::Fatal);
        assert!(runner.commands.is_empty());
    }

    #[test]
    fn the_worst_verdict_is_what_decides() {
        let checks = vec![
            Check::ok("a"),
            Check::warn("b", "x"),
            Check::fatal("c", "y"),
        ];
        assert_eq!(worst(&checks), Verdict::Fatal);
        assert_eq!(worst(&checks[..2]), Verdict::Warn);
        assert_eq!(worst(&checks[..1]), Verdict::Ok);
    }
}
