//! The commands themselves.

use std::path::Path;

use anyhow::{Result, bail};
use console::style;

use crate::checks::{self, Verdict};
use crate::config::{Answers, Discord, generated_password};
use crate::envfile;
use crate::exec::Runner;
use crate::stack;
use crate::ui;

const ENV: &str = ".env";
const EXAMPLE: &str = ".env.example";

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

    ui::heading("Checking the network");
    let mut fatal = false;
    let ours = stack::proxy_running(runner, dir);
    for check in checks::ports(&[answers.http_port, answers.https_port], ours) {
        ui::report(&check);
        fatal |= check.verdict == Verdict::Fatal;
    }
    let dns = checks::dns(&answers.domain, checks::public_ip());
    ui::report(&dns);
    fatal |= dns.verdict == Verdict::Fatal;
    if fatal {
        bail!("sort the above out and run this again");
    }

    write_env(dir, &template(dir), &answers)?;
    ui::done("wrote .env");

    ui::heading("Building");
    stack::build(runner, dir)?;
    ui::heading("Starting");
    stack::up(runner, dir)?;

    after_boot(&answers);
    Ok(())
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

    // Migrations run on boot, so a rebuild and a restart is the whole of it.
    stack::build(runner, dir)?;
    stack::up(runner, dir)?;
    ui::done("running");
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
    let domain = envfile::get(&env, "WS_DOMAIN").unwrap_or_default();
    if !domain.is_empty() {
        ui::heading("Public URL");
        let url = format!("https://{domain}");
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
    let domain = envfile::get(&env, "WS_DOMAIN").unwrap_or_default();
    let (http, https) = ports(&env, &domain);

    ui::heading("This machine");
    let checks = checks::all(runner, dir, &domain, http, https)?;
    for check in &checks {
        ui::report(check);
    }
    if checks::worst(&checks) == Verdict::Fatal {
        bail!("something above has to be dealt with first");
    }
    Ok(())
}

fn ask_everything(existing: &str, http: u16, https: u16) -> Result<Answers> {
    ui::heading("What this deployment needs");
    let keep = |key: &str| envfile::get(existing, key).filter(|v| !v.is_empty());

    ui::note("Leave the domain blank to serve plain http on this machine.");
    let domain = normalise_domain(&ui::ask_optional("Public domain", keep("WS_DOMAIN"))?);
    let (http_port, https_port) = if domain.is_empty() {
        (http, https)
    } else {
        (80, 443)
    };

    let mut answers = Answers {
        domain,
        http_port,
        https_port,
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

    answers.discord = ask_discord(existing, &answers.base_url())?;
    Ok(answers)
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
    ui::heading("After the first boot");
    ui::note("The API downloads CCP's static data and seeds from it, which takes a few");
    ui::note("minutes. Follow it with:  docker compose logs -f api");
    if answers.discord.is_some() {
        println!();
        ui::note("Then finish Discord, which needs this running to verify itself:");
        ui::note(&format!(
            "  1. Set the Interactions Endpoint URL to {}/discord/interactions.",
            answers.base_url()
        ));
        ui::note("     Discord signs a ping at it and refuses to save if it does not answer.");
        ui::note("  2. wsctl discord-register");
    }
    println!(
        "\n  Then open {}",
        style(answers.base_url()).cyan().underlined()
    );
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

fn write_env(dir: &Path, existing: &str, answers: &Answers) -> Result<()> {
    let mut values = answers.env_values();
    // A database nobody chose a password for is a database with the password in the README.
    if envfile::get(existing, "POSTGRES_PASSWORD")
        .unwrap_or_default()
        .is_empty()
    {
        values.insert("POSTGRES_PASSWORD".into(), generated_password());
    }

    let mut out = envfile::patch(existing, &values);
    for key in answers.env_removals() {
        out = envfile::remove(&out, key);
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

#[cfg(test)]
mod tests {
    use super::*;

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
