//! The commands themselves.

use std::path::Path;

use anyhow::{Result, bail};
use console::style;

use crate::checks::{self, Verdict};
use crate::config::{
    self, Answers, DEFAULT_NETWORK, DEFAULT_ROUTER_PORT, Database, Deployment, Direct, Discord,
    ExternalDb, Images, Proxy, Router, Site, Tls, generated_password,
};
use crate::envfile;
use crate::exec::Runner;
use crate::integrate;
use crate::rename::{self, Renamed};
use crate::stack;
use crate::ui;

const ENV: &str = ".env";
const EXAMPLE: &str = ".env.example";
/// What the compose file fell back to for the database password before wsctl 0.5.
const LEGACY_PASSWORD: &str = "vector";

/// The compose file is what every command here drives, so its absence is the one thing
/// worth saying plainly rather than letting docker fail three steps later.
fn require_checkout(dir: &Path) -> Result<()> {
    if dir.join("docker-compose.yml").is_file() {
        return Ok(());
    }
    bail!(
        "no docker-compose.yml in {}.\n  Run this inside a WormholeSystems checkout:\n    git clone git@github.com:WormholeSystems/wormholesystems-next.git\n    cd wormholesystems-next && wsctl setup\n  Or point at one with --dir.",
        dir.display()
    )
}

/// The prompts need a terminal to draw on. Without one they would block forever waiting
/// for a keystroke that cannot arrive, which is what running this from a script looks like.
fn require_terminal() -> Result<()> {
    if console::user_attended() {
        return Ok(());
    }
    bail!(
        "setup asks questions, so it needs a terminal. Run it from a shell, or edit .env by hand."
    )
}

pub fn setup(runner: &mut dyn Runner, dir: &Path) -> Result<()> {
    require_checkout(dir)?;
    require_terminal()?;
    ui::heading("Checking this machine");
    let existing = read_env(dir);
    let domain = envfile::get(&existing, "WS_DOMAIN").unwrap_or_default();
    let (http, https) = ports(&existing, &domain);

    // Docker first and on its own: without it nothing else can be answered, and the port
    // check would report ports as free that its own stack is holding.
    let docker = checks::docker(runner, dir);
    ui::report(&docker);
    if docker.verdict == Verdict::Fatal {
        bail!("docker is not usable yet");
    }
    for check in [checks::disk(dir), checks::repo(runner, dir)] {
        ui::report(&check);
    }

    let answers = ask_everything(&existing, http, https)?;
    let deployment = &answers.deployment;

    ui::heading("Checking the network");
    if let Some(network) = deployment.proxy_network()
        && stack::ensure_network(runner, dir, network)?
    {
        ui::done(&format!("created the Docker network {network}"));
    }
    let results = checks::deployment(runner, dir, deployment);
    for check in &results {
        ui::report(check);
    }
    if checks::worst(&results) == Verdict::Fatal {
        bail!("sort the above out and run this again");
    }
    let leftovers = leftovers(runner, dir, deployment);

    // Before .env names the new project, which is what the old name is read from. A fresh
    // checkout has no .env yet, and nothing under any name to move.
    let bundled = deployment.database == Database::Bundled;
    let (old, has_database) = if existing.is_empty() {
        (None, false)
    } else {
        let old = rename::old_project(runner, dir)?;
        let has_database = rename::has_database(runner, dir, old.as_deref())?;
        (old, has_database)
    };

    write_env(dir, &template(dir), &answers, bundled && has_database)?;
    ui::done("wrote .env");

    // Ready before anything is stopped, so a running install only goes down for the move.
    ui::heading(match answers.images {
        Images::Pull => "Pulling the images",
        Images::Build => "Building",
    });
    stack::images(runner, dir, answers.images)?;
    let mut renamed = match &old {
        Some(old) => rename::move_project(runner, dir, old)?,
        None => Renamed::default(),
    };
    if bundled {
        renamed.role = rename::rename_role(runner, dir)?;
    }
    report_renamed(&renamed);
    ui::heading("Starting");
    if existing.is_empty() {
        ui::note("The first boot downloads CCP's static data (~550MB) and seeds from it before");
        ui::note("the API answers. This waits for that, which takes a few minutes; follow it");
        ui::note("from another shell with:  docker compose logs -f api");
    }
    stack::up(runner, dir, answers.images)?;
    ui::done("running and healthy");

    after_boot(&answers);
    report_leftovers(&leftovers);
    Ok(())
}

/// The bundled services this install no longer uses but that are still up. Compose leaves
/// them be (an overlay takes them out of the stack, not off the machine), and the operator
/// decides when the old database is no longer needed.
fn leftovers(runner: &mut dyn Runner, dir: &Path, deployment: &Deployment) -> Vec<&'static str> {
    let mut out = Vec::new();
    if deployment.database != Database::Bundled && stack::service_running(runner, dir, "db") {
        out.push("db");
    }
    if matches!(deployment.proxy, Proxy::Direct(_)) && stack::proxy_running(runner, dir) {
        out.push("caddy");
    }
    out
}

fn report_leftovers(leftovers: &[&str]) {
    if leftovers.is_empty() {
        return;
    }
    println!();
    let what = match leftovers {
        ["db"] => "Postgres is still running, holding the old database",
        ["caddy"] => "Caddy is still running, holding ports 80 and 443",
        _ => "Postgres and Caddy are still running, holding the old database and ports 80 and 443",
    };
    ui::note(&format!("The bundled {what}."));
    ui::note("Once nothing needs them any more (moving the data is in docs/deployment.md):");
    ui::note(&format!("  docker compose rm -sf {}", leftovers.join(" ")));
}

/// Bring the checkout, the containers and the static data up to date. One command,
/// because a version of the code and the export it reads are not separately interesting:
/// whoever is updating the server wants the server updated.
pub fn update(runner: &mut dyn Runner, dir: &Path, force_sde: bool) -> Result<()> {
    require_checkout(dir)?;
    ui::heading("Updating");
    let before = runner
        .capture(dir, "git", &["rev-parse", "--short", "HEAD"])
        .unwrap_or_default();
    if runner.run(dir, "git", &["pull", "--ff-only"]).is_err() {
        bail!("could not fast-forward the checkout; sort the working tree out first");
    }
    let after = runner
        .capture(dir, "git", &["rev-parse", "--short", "HEAD"])
        .unwrap_or_default();
    if before == after {
        ui::done(&format!("already on {after}"));
    } else {
        ui::done(&format!("{before} → {after}"));
    }

    let env = read_env(dir);
    let deployment = Deployment::from_env(&env);
    let bundled = deployment.database == Database::Bundled;
    let old = rename::old_project(runner, dir)?;
    let mut values = std::collections::BTreeMap::new();
    if old.is_some() {
        values.insert(
            "COMPOSE_PROJECT_NAME".to_string(),
            rename::PROJECT.to_string(),
        );
    }
    // The old compose file fell back to `vector` for a password nobody set, and the
    // database still has it; the new fallback would not match.
    if bundled
        && envfile::get(&env, "POSTGRES_PASSWORD")
            .unwrap_or_default()
            .is_empty()
        && rename::has_database(runner, dir, old.as_deref())?
    {
        values.insert("POSTGRES_PASSWORD".to_string(), LEGACY_PASSWORD.to_string());
    }
    // Written out every time rather than only when something is missing: a .env from
    // wsctl 0.5 moves from WS_SETUP onto the two choices and their overlays, and
    // COMPOSE_FILE follows whatever else changed.
    let rewritten = deployment.apply(&envfile::patch(&env, &values));
    if rewritten != env {
        std::fs::write(dir.join(ENV), &rewritten)?;
    }
    if Deployment::is_legacy(&env) {
        ui::done(&format!(
            "moved .env from WS_SETUP to WS_DATABASE={} and WS_PROXY={}",
            envfile::get(&rewritten, "WS_DATABASE").unwrap_or_default(),
            envfile::get(&rewritten, "WS_PROXY").unwrap_or_default(),
        ));
    }

    // Migrations run on boot, so new images and a restart is the whole of it. The images
    // come first, while the old containers keep serving.
    let images = Images::from_env(&env);
    if !Images::chosen(&env) {
        ui::note("This install builds its images here. Prebuilt ones for amd64 and arm64 are");
        ui::note("published for every push to main; WS_IMAGES=pull in .env (or wsctl setup)");
        ui::note("switches to them and skips the compile.");
    }
    stack::images(runner, dir, images)?;
    let mut renamed = match &old {
        Some(old) => rename::move_project(runner, dir, old)?,
        None => Renamed::default(),
    };
    if bundled {
        renamed.role = rename::rename_role(runner, dir)?;
    }
    report_renamed(&renamed);
    stack::up(runner, dir, images)?;
    ui::done("running and healthy");
    // Before the static data, which wants the space a stale cache is sitting on.
    match stack::prune(runner, dir) {
        Ok(()) => ui::done("old build cache pruned"),
        Err(err) => println!(
            "  {} could not prune the old build cache: {err:#}",
            style("!").yellow()
        ),
    }
    update_sde(runner, dir, force_sde);
    Ok(())
}

/// Say what moved, and what is left behind to clean up once the new names are running.
fn report_renamed(renamed: &Renamed) {
    if let Some(old) = &renamed.from_project {
        ui::done(&format!(
            "moved the stack from project {old} to {}",
            rename::PROJECT
        ));
    }
    if renamed.role {
        ui::done(&format!(
            "renamed the database role and database to {}",
            rename::PROJECT
        ));
    }
    for volume in &renamed.skipped {
        println!(
            "  {} kept the existing {}_* volume instead of copying {volume}",
            style("!").yellow(),
            rename::PROJECT
        );
    }
    if !renamed.copied.is_empty() {
        ui::note("The old volumes are still there, untouched. Once the new stack looks right:");
        ui::note(&format!("  docker volume rm {}", renamed.copied.join(" ")));
    }
}

/// What `status` says about the static data. Reports and changes nothing, unlike its
/// namesake in `update`: asking what is running should never start a download.
fn report_sde(runner: &mut dyn Runner, dir: &Path) {
    match stack::sde_status(runner, dir) {
        Ok(sde) if sde.update_available => println!(
            "  {} a newer build is out ({}); `wsctl update` takes it",
            style("!").yellow(),
            sde.latest
        ),
        Ok(sde) => ui::done(&format!("current ({})", sde.loaded)),
        Err(_) => println!(
            "  {} could not read it; is the stack built?",
            style("!").yellow()
        ),
    }
}

/// Take a newer export when CCP has one, and say so either way. Failing to read the status
/// does not fail the update: the code is already live, and the static data can wait.
fn update_sde(runner: &mut dyn Runner, dir: &Path, force: bool) {
    let status = match stack::sde_status(runner, dir) {
        Ok(status) => status,
        Err(_) => {
            println!(
                "  {} could not read the static data; is the stack built?",
                style("!").yellow()
            );
            return;
        }
    };
    if !status.update_available && !force {
        ui::done(&format!("static data current ({})", status.loaded));
        return;
    }

    ui::note("A few hundred MB of static data, then a re-seed. This takes a while.");
    let fetched = stack::api(runner, dir, &["sde-fetch", "--force"])
        .and_then(|_| stack::restart(runner, dir, "api"));
    match fetched {
        Ok(()) => ui::done(&format!(
            "static data {} → {}; the API re-seeds from it as it boots",
            status.loaded, status.latest
        )),
        Err(err) => println!(
            "  {} static data left at {}: {err:#}",
            style("!").yellow(),
            status.loaded
        ),
    }
}

pub fn status(runner: &mut dyn Runner, dir: &Path) -> Result<()> {
    require_checkout(dir)?;
    ui::heading("Containers");
    println!("{}", stack::ps(runner, dir)?);

    ui::heading("Checkout");
    {
        let check = checks::repo(runner, dir);
        ui::report(&check);
    }

    ui::heading("Static data");
    report_sde(runner, dir);

    let env = read_env(dir);
    if let Some(url) = Deployment::from_env(&env).public_url() {
        ui::heading("Public URL");
        match runner.capture(
            dir,
            "curl",
            &[
                "-fsS",
                "-o",
                "/dev/null",
                "-w",
                "%{http_code}",
                "--max-time",
                "10",
                &url,
            ],
        ) {
            Ok(code) if code.starts_with('2') || code.starts_with('3') => {
                ui::done(&format!("{url} answers ({code})"))
            }
            Ok(code) => println!("  {} {url} answered {code}", style("!").yellow()),
            Err(_) => println!("  {} {url} did not answer", style("✗").red()),
        }
    }
    Ok(())
}

pub fn discord_register(runner: &mut dyn Runner, dir: &Path) -> Result<()> {
    require_checkout(dir)?;
    let env = read_env(dir);
    if envfile::get(&env, "DISCORD_BOT_TOKEN")
        .unwrap_or_default()
        .is_empty()
    {
        bail!("DISCORD_BOT_TOKEN is not set; the slash command needs the bot");
    }
    stack::api(runner, dir, &["discord-register"])?;
    ui::done("uploaded; Discord takes a few minutes to roll it out");
    Ok(())
}

pub fn doctor(runner: &mut dyn Runner, dir: &Path) -> Result<()> {
    let env = read_env(dir);
    let deployment = Deployment::from_env(&env);

    ui::heading("This machine");
    let mut checks = checks::machine(runner, dir);
    checks.extend(checks::deployment(runner, dir, &deployment));
    for check in &checks {
        ui::report(check);
    }
    if checks::worst(&checks) == Verdict::Fatal {
        bail!("something above has to be dealt with first");
    }
    Ok(())
}

/// What a reverse proxy in front of this install needs, again, for whoever sets it up.
pub fn integrate(dir: &Path) -> Result<()> {
    require_checkout(dir)?;
    integrate::print(&Deployment::from_env(&read_env(dir)));
    Ok(())
}

fn ask_everything(existing: &str, http: u16, https: u16) -> Result<Answers> {
    ui::heading("What this deployment needs");
    let keep = |key: &str| envfile::get(existing, key).filter(|v| !v.is_empty());

    let previous = Deployment::from_env(existing);
    let database = ask_database(&previous)?;
    let proxy = ask_proxy(existing, &previous.proxy, http, https)?;
    let mut answers = Answers {
        deployment: Deployment { database, proxy },
        ..Default::default()
    };

    println!();
    ui::note("Every request to ESI, zKillboard and EVE Ref carries these, so those services");
    ui::note("can tell operators apart and reach you if this install misbehaves. CCP asks");
    ui::note("for it; anonymous clients get throttled, then blocked.");
    answers.contact_name = ui::ask(
        "In-game character name of the admin",
        keep("WS_CONTACT_NAME"),
    )?;
    answers.contact_email = ui::ask("Contact email", keep("WS_CONTACT_EMAIL"))?;

    println!();
    ui::note("EVE application, from https://developers.eveonline.com/applications.");
    ui::note(&format!(
        "Its callback URL must be exactly: {}/auth/callback",
        answers.base_url()
    ));
    answers.eve_client_id = ui::ask("Client ID", keep("EVE_CLIENT_ID"))?;
    answers.eve_client_secret = ui::ask_secret("Client secret", keep("EVE_CLIENT_SECRET"))?;

    println!();
    ui::note("Who may log in: EVE character, corporation and alliance ids, comma separated.");
    ui::note("A pilot gets in when their character, corporation or alliance is listed.");
    ui::note("Leave it empty to let anyone with an EVE account in.");
    answers.allowed_logins = ui::edit(
        "Allowed ids",
        &envfile::get(existing, "ALLOWED_AFFILIATION_IDS").unwrap_or_default(),
        config::affiliation_ids,
    )?;
    ui::note("Who may create maps, the same way, checked against the character a map is");
    ui::note("created as. Empty lets everyone who can log in.");
    answers.map_creators = ui::edit(
        "Map creator ids",
        &envfile::get(existing, "MAP_CREATOR_AFFILIATION_IDS").unwrap_or_default(),
        config::affiliation_ids,
    )?;

    answers.discord = ask_discord(existing, &answers.base_url())?;
    answers.images = ask_images(existing)?;
    Ok(answers)
}

fn ask_database(previous: &Deployment) -> Result<Database> {
    println!();
    ui::note("Postgres: one in a container here, with its data in a Docker volume, or one you");
    ui::note("already run for other things.");
    let external = matches!(previous.database, Database::External(_));
    let choice = ui::choose(
        "Database",
        &["Run one here", "Use one I already have"],
        usize::from(external),
    )?;
    if choice == 0 {
        return Ok(Database::Bundled);
    }
    let before = match &previous.database {
        Database::External(db) => db.clone(),
        Database::Bundled => ExternalDb::default(),
    };

    println!();
    ui::note("The Postgres to use, as the containers reach it (not localhost: that is the");
    ui::note("container itself). The database has to exist; the API creates its tables.");
    let url = ui::ask_secret("Database URL", Some(before.url).filter(|u| !u.is_empty()))?;

    println!();
    ui::note("When it is a container on a Docker network, the API joins that network to");
    ui::note("reach it by name. Leave it empty for a database reachable by its address.");
    // An install from before this was asked reached its database over the proxy's network.
    let network = before
        .network
        .or_else(|| previous.app_network().map(String::from))
        .unwrap_or_default();
    let network = ui::edit("Its Docker network", &network, tidy)?;
    Ok(Database::External(ExternalDb {
        url,
        network: Some(network).filter(|n| !n.is_empty()),
    }))
}

fn ask_proxy(existing: &str, previous: &Proxy, http: u16, https: u16) -> Result<Proxy> {
    println!();
    ui::note("What faces the internet. Caddy here can take ports 80 and 443 and get the");
    ui::note("certificate itself, or sit behind a proxy you already run and split the paths");
    ui::note("for it. Or your proxy splits them itself and reaches the API and the frontend");
    ui::note("directly, over a Docker network it shares with them or on loopback ports.");
    let current = match previous {
        Proxy::Caddy(_) => 0,
        Proxy::Router(_) => 1,
        Proxy::Direct(Direct {
            network: Some(_), ..
        }) => 2,
        Proxy::Direct(_) => 3,
    };
    let choice = ui::choose(
        "Proxy",
        &[
            "Caddy here, with its own certificate",
            "Caddy here, behind my proxy, which sends it the whole site",
            "My proxy, in Docker, on a network shared with the app",
            "My proxy, on this machine, on loopback ports",
        ],
        current,
    )?;
    let keep = |key: &str| envfile::get(existing, key).filter(|v| !v.is_empty());
    match choice {
        0 => ask_site(existing, http, https).map(Proxy::Caddy),
        1 => ask_router(keep("WS_PUBLIC_URL"), previous).map(Proxy::Router),
        n => ask_direct(keep("WS_PUBLIC_URL"), previous, n == 2).map(Proxy::Direct),
    }
}

fn ask_site(existing: &str, http: u16, https: u16) -> Result<Site> {
    let keep = |key: &str| envfile::get(existing, key).filter(|v| !v.is_empty());
    println!();
    ui::note("Leave the domain blank to serve plain http on this machine.");
    let domain = normalise_domain(&ui::ask_optional("Public domain", keep("WS_DOMAIN"))?);
    if !domain.is_empty() {
        let tls = ask_tls(existing)?;
        // Let's Encrypt only ever looks at 80 and 443. A certificate from anywhere else
        // can be served wherever this install already serves it.
        let (http_port, https_port) = if tls == Tls::Acme {
            (80, 443)
        } else {
            let read = |key, fallback| {
                envfile::get(existing, key)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(fallback)
            };
            (read("HTTP_PORT", 80), read("HTTPS_PORT", 443))
        };
        return Ok(Site {
            domain,
            http_port,
            https_port,
            public_url: String::new(),
            tls,
        });
    }

    println!();
    ui::note("Without a domain the EVE and Discord callbacks are built from the address people");
    ui::note("open it at: localhost on this machine, this server's address from anywhere else.");
    let suggested = keep("WS_PUBLIC_URL").unwrap_or_else(|| suggested_url(http));
    let public_url = normalise_plain_url(&ui::ask("Public URL", Some(suggested))?);
    Ok(Site {
        domain,
        http_port: http,
        https_port: https,
        public_url,
        tls: Tls::Acme,
    })
}

fn ask_tls(existing: &str) -> Result<Tls> {
    let current = envfile::get(existing, "WS_TLS")
        .and_then(|v| Tls::parse(&v))
        .unwrap_or_default();
    println!();
    ui::note("Where the certificate comes from. Let's Encrypt needs the name to resolve here");
    ui::note("and ports 80 and 443 open to the internet. Your own goes in certs/ in this");
    ui::note("checkout, as fullchain.pem and privkey.pem.");
    let choice = ui::choose(
        "Certificate",
        &[
            "Let's Encrypt",
            "My own, from certs/",
            "Caddy's internal CA (browsers warn; for a private network)",
            "None: plain http, something in front does TLS",
        ],
        Tls::ALL.iter().position(|t| *t == current).unwrap_or(0),
    )?;
    Ok(Tls::ALL[choice])
}

fn ask_router(public_url: Option<String>, previous: &Proxy) -> Result<Router> {
    let before = match previous {
        Proxy::Router(router) => router.clone(),
        Proxy::Direct(direct) => Router {
            network: direct.network.clone(),
            ..Default::default()
        },
        Proxy::Caddy(_) => Router::default(),
    };
    let public_url = ask_public_url(public_url)?;

    println!();
    ui::note("Your proxy reaches this Caddy over a Docker network they share, as");
    ui::note("wormholesystems-router:80, or on a loopback port when it runs on this machine.");
    let in_docker = ui::choose(
        "Your proxy runs",
        &["in Docker", "on this machine"],
        usize::from(before.network.is_none() && before.port.is_some()),
    )? == 0;
    let (network, port) = if in_docker {
        (Some(ask_network(before.network)?), None)
    } else {
        let port = ask_port("Loopback port", before.port.unwrap_or(DEFAULT_ROUTER_PORT))?;
        (None, Some(port))
    };
    Ok(Router {
        public_url,
        network,
        port,
    })
}

fn ask_direct(public_url: Option<String>, previous: &Proxy, in_docker: bool) -> Result<Direct> {
    let before = match previous {
        Proxy::Direct(direct) => direct.clone(),
        Proxy::Router(router) => Direct {
            network: router.network.clone(),
            ..Default::default()
        },
        Proxy::Caddy(_) => Direct::default(),
    };
    let public_url = ask_public_url(public_url)?;

    let network = if in_docker {
        println!();
        ui::note("Your proxy finds the app on this network as wormholesystems-api and");
        ui::note("wormholesystems-web.");
        Some(ask_network(before.network)?)
    } else {
        None
    };

    println!();
    let host_ports = if !in_docker
        || ui::confirm(
            "Also publish on 127.0.0.1, for a proxy running on this machine rather than in Docker?",
            before.host_ports.is_some(),
        )? {
        let (api, web) = before.host_ports.unwrap_or((3000, 3001));
        Some((ask_port("API port", api)?, ask_port("Frontend port", web)?))
    } else {
        None
    };

    Ok(Direct {
        public_url,
        network,
        host_ports,
    })
}

fn ask_public_url(previous: Option<String>) -> Result<String> {
    println!();
    ui::note("The address people open, as your proxy serves it. The EVE and Discord callbacks");
    ui::note("are built from it.");
    Ok(normalise_url(&ui::ask("Public URL", previous)?))
}

fn ask_network(previous: Option<String>) -> Result<String> {
    ui::note("It is created if it does not exist.");
    ui::ask(
        "Docker network",
        previous.or_else(|| Some(DEFAULT_NETWORK.into())),
    )
}

/// What to offer as the address of a Caddy without a domain. Over SSH the browser is
/// somewhere else, and the address this machine was reached at is the likely answer.
fn suggested_url(http_port: u16) -> String {
    let server = std::env::var("SSH_CONNECTION")
        .ok()
        .and_then(|c| c.split_whitespace().nth(2).map(String::from));
    match server {
        Some(ip) => {
            let host = if ip.contains(':') {
                format!("[{ip}]")
            } else {
                ip
            };
            if http_port == 80 {
                format!("http://{host}")
            } else {
                format!("http://{host}:{http_port}")
            }
        }
        None => config::localhost(http_port),
    }
}

fn tidy(input: &str) -> Result<String, String> {
    Ok(input.trim().to_string())
}

/// Pulled for a new install. One that has been building keeps the answer it had until its
/// operator picks the other here.
fn ask_images(existing: &str) -> Result<Images> {
    let current = if existing.is_empty() {
        Images::Pull
    } else {
        Images::from_env(existing)
    };
    println!();
    ui::note("The API and the frontend are published as images for amd64 and arm64, built from");
    ui::note("every push to main. Building them here instead compiles the Rust API on this");
    ui::note("machine: about 2GB of memory and ten minutes or more on a small server.");
    let choice = ui::choose(
        "Images",
        &[
            "Pull the published images",
            "Build them here from this checkout",
        ],
        usize::from(current == Images::Build),
    )?;
    Ok(if choice == 0 {
        Images::Pull
    } else {
        Images::Build
    })
}

fn ask_port(prompt: &str, default: u16) -> Result<u16> {
    loop {
        match ui::ask(prompt, Some(default.to_string()))?.parse() {
            Ok(port) if port > 0 => return Ok(port),
            _ => ui::note("a port number, 1 to 65535"),
        }
    }
}

fn ask_discord(existing: &str, base: &str) -> Result<Option<Discord>> {
    println!();
    let already = envfile::get(existing, "DISCORD_CLIENT_ID")
        .filter(|v| !v.is_empty())
        .is_some();
    if !ui::confirm(
        "Set up Discord? (account linking, slash commands, alerts to a channel)",
        already,
    )? {
        return Ok(None);
    }

    println!();
    ui::note("At https://discord.com/developers/applications, make an application, then:");
    ui::note("  General Information  the Application ID and the Public Key");
    ui::note("  OAuth2               the Client ID and Client Secret, and add this redirect");
    ui::note(&format!(
        "                       exactly: {base}/discord/callback"
    ));
    ui::note("  Bot                  only to post as a bot or send DMs; leave blank otherwise");
    println!();

    let keep = |key: &str| envfile::get(existing, key).filter(|v| !v.is_empty());
    Ok(Some(Discord {
        application_id: ui::ask("Application ID", keep("DISCORD_APPLICATION_ID"))?,
        public_key: ui::ask("Public key", keep("DISCORD_PUBLIC_KEY"))?,
        client_id: ui::ask("Client ID", keep("DISCORD_CLIENT_ID"))?,
        client_secret: ui::ask_secret("Client secret", keep("DISCORD_CLIENT_SECRET"))?,
        bot_token: ui::ask_secret("Bot token (optional)", keep("DISCORD_BOT_TOKEN"))?,
    }))
}

fn after_boot(answers: &Answers) {
    if answers.discord.is_some() {
        ui::heading("Finishing Discord");
        ui::note("It needs the stack running to verify itself, which it now is:");
        ui::note(&format!(
            "  1. Set the Interactions Endpoint URL to {}/discord/interactions.",
            answers.base_url()
        ));
        ui::note("     Discord signs a ping at it and refuses to save if it does not answer.");
        ui::note("  2. wsctl discord-register");
    }
    if !matches!(answers.deployment.proxy, Proxy::Caddy(_)) {
        integrate::print(&answers.deployment);
        println!();
        ui::note("This is printed again by:  wsctl integrate");
    }
    println!("\n  Open {}", style(answers.base_url()).cyan().underlined());
}

/// What is already configured. Only a real `.env` counts: the example file is a template
/// full of placeholders, and offering `your-client-id` as the default answer invites
/// someone to press enter straight past it.
fn read_env(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(ENV)).unwrap_or_default()
}

/// What a new `.env` is written on top of, so the comments explaining each setting come
/// with it.
fn template(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(ENV))
        .or_else(|_| std::fs::read_to_string(dir.join(EXAMPLE)))
        .unwrap_or_default()
}

fn write_env(dir: &Path, existing: &str, answers: &Answers, has_database: bool) -> Result<()> {
    let mut out = answers.apply(existing);
    // A database nobody chose a password for is a database with the password in the README.
    // An external one has no container here to set one for. One that already exists was
    // made with the old fallback, and a new password in .env would not change it.
    if answers.deployment.database == Database::Bundled
        && envfile::get(existing, "POSTGRES_PASSWORD")
            .unwrap_or_default()
            .is_empty()
    {
        let password = if has_database {
            LEGACY_PASSWORD.to_string()
        } else {
            generated_password()
        };
        let values = [("POSTGRES_PASSWORD".to_string(), password)].into();
        out = envfile::patch(&out, &values);
    }

    let path = dir.join(ENV);
    std::fs::write(&path, out)?;
    // It holds the client secret and the database password.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Ports only matter without a domain; with one it is 80 and 443, because that is where
/// Let's Encrypt looks.
fn ports(env: &str, domain: &str) -> (u16, u16) {
    if domain.is_empty() {
        let read = |key, fallback| {
            envfile::get(env, key)
                .and_then(|v| v.parse().ok())
                .unwrap_or(fallback)
        };
        (read("HTTP_PORT", 8080), read("HTTPS_PORT", 8443))
    } else {
        (80, 443)
    }
}

/// A bare hostname is what Caddy and the redirect URLs want, but a pasted address usually
/// brings a scheme and a trailing slash with it.
fn normalise_domain(input: &str) -> String {
    input
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

/// A public URL with its scheme and without a trailing slash. A bare hostname gets https,
/// since a proxy that is good enough to sit in front of this is serving https.
fn normalise_url(input: &str) -> String {
    let url = input.trim().trim_end_matches('/');
    if url.starts_with("https://") || url.starts_with("http://") {
        url.to_string()
    } else {
        format!("https://{url}")
    }
}

/// The same for a Caddy without a domain, which serves plain http, so a bare address gets
/// that.
fn normalise_plain_url(input: &str) -> String {
    let url = input.trim().trim_end_matches('/');
    if url.starts_with("https://") || url.starts_with("http://") {
        url.to_string()
    } else {
        format!("http://{url}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::Recording;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A checkout of its own for one test: a compose file to find and the given `.env`.
    /// Removed again when the test is done with it.
    struct Checkout(PathBuf);

    impl std::ops::Deref for Checkout {
        type Target = Path;
        fn deref(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Checkout {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn checkout(env: &str) -> Checkout {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "wsctl-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("docker-compose.yml"), "services: {}\n").unwrap();
        std::fs::write(dir.join(ENV), env).unwrap();
        Checkout(dir)
    }

    /// Docker as an install that already runs under the current names answers it.
    fn running() -> Recording {
        let mut runner = Recording::default();
        runner.replies.insert(
            "docker compose config".into(),
            "name: wormholesystems\n".into(),
        );
        runner
    }

    const INSTALLED: &str = "COMPOSE_PROJECT_NAME=wormholesystems\nPOSTGRES_PASSWORD=secret\n";

    #[test]
    fn an_update_that_pulls_never_builds() {
        let dir = checkout(&format!("{INSTALLED}WS_IMAGES=pull\n"));
        let mut runner = running();
        update(&mut runner, &dir, false).unwrap();
        assert!(runner.commands.contains(&"docker compose pull".to_string()));
        assert!(
            runner.commands.contains(
                &"docker compose up -d --wait --wait-timeout 1800 --no-build".to_string()
            )
        );
        assert!(!runner.commands.iter().any(|c| c.contains("compose build")));
    }

    // An install from before the choice has always built, and keeps doing so.
    #[test]
    fn an_update_that_never_chose_builds() {
        let dir = checkout(INSTALLED);
        let mut runner = running();
        update(&mut runner, &dir, false).unwrap();
        let build = runner
            .commands
            .iter()
            .position(|c| c == "docker compose build")
            .expect("built");
        let up = runner
            .commands
            .iter()
            .position(|c| c == "docker compose up -d --wait --wait-timeout 1800")
            .expect("started");
        assert!(build < up);
        assert!(!runner.commands.contains(&"docker compose pull".to_string()));
    }

    // Production runs exactly this: a standalone install from wsctl 0.5 behind Coolify.
    // The update has to keep it the same stack, only spelled with the overlays.
    #[test]
    fn an_update_moves_a_standalone_env_onto_the_overlays() {
        let dir = checkout(
            "COMPOSE_PROJECT_NAME=wormholesystems\nWS_SETUP=standalone\nCOMPOSE_FILE=compose.standalone.yml\nWS_PUBLIC_URL=https://map.example.com\nWS_DATABASE_URL=postgres://ws:secret@postgres:5432/ws\nWS_PROXY_NETWORK=coolify\n",
        );
        let mut runner = running();
        update(&mut runner, &dir, false).unwrap();
        let env = read_env(&dir);
        assert_eq!(
            envfile::get(&env, "COMPOSE_FILE").as_deref(),
            Some("docker-compose.yml:deploy/external-db.yml:deploy/app-network.yml")
        );
        assert_eq!(
            envfile::get(&env, "WS_DATABASE").as_deref(),
            Some("external")
        );
        assert_eq!(envfile::get(&env, "WS_PROXY").as_deref(), Some("network"));
        assert_eq!(
            envfile::get(&env, "WS_PROXY_NETWORK").as_deref(),
            Some("coolify")
        );
        assert_eq!(envfile::get(&env, "WS_SETUP"), None);
        // Nothing of the bundled database is touched: there is none.
        assert!(!runner.commands.iter().any(|c| c.contains("psql")));
        assert!(!runner.commands.iter().any(|c| c.contains("volume ls")));
    }

    // A bundled install gets the two keys written and no COMPOSE_FILE: compose's default
    // is already the whole stack.
    #[test]
    fn an_update_leaves_a_bundled_install_on_the_default_file() {
        let dir = checkout(&format!("{INSTALLED}WS_DOMAIN=map.example.com\n"));
        update(&mut running(), &dir, false).unwrap();
        let env = read_env(&dir);
        assert_eq!(envfile::get(&env, "COMPOSE_FILE"), None);
        assert_eq!(envfile::get(&env, "WS_PROXY").as_deref(), Some("caddy"));
        assert_eq!(
            envfile::get(&env, "WS_DOMAIN").as_deref(),
            Some("map.example.com")
        );
    }

    #[test]
    fn a_public_url_keeps_its_scheme_and_loses_the_trailing_slash() {
        assert_eq!(
            normalise_url("https://map.example.com/"),
            "https://map.example.com"
        );
        assert_eq!(
            normalise_url("http://localhost:8080"),
            "http://localhost:8080"
        );
        assert_eq!(
            normalise_url(" map.example.com "),
            "https://map.example.com"
        );
    }

    #[test]
    fn a_bare_address_without_a_domain_is_plain_http() {
        assert_eq!(
            normalise_plain_url("203.0.113.5:8080/"),
            "http://203.0.113.5:8080"
        );
        assert_eq!(
            normalise_plain_url("https://tunnel.example"),
            "https://tunnel.example"
        );
    }

    #[test]
    fn a_pasted_address_is_reduced_to_the_hostname() {
        for input in [
            "https://map.example.com/",
            "http://map.example.com",
            "map.example.com",
            "  map.example.com  ",
        ] {
            assert_eq!(
                normalise_domain(input),
                "map.example.com",
                "input {input:?}"
            );
        }
        assert_eq!(normalise_domain(""), "");
    }

    #[test]
    fn a_domain_pins_the_ports_lets_encrypt_looks_at() {
        assert_eq!(ports("HTTP_PORT=9000\n", "map.example.com"), (80, 443));
    }

    #[test]
    fn without_a_domain_the_ports_are_whatever_was_chosen() {
        assert_eq!(ports("HTTP_PORT=9000\nHTTPS_PORT=9443\n", ""), (9000, 9443));
        assert_eq!(ports("", ""), (8080, 8443));
    }
}
