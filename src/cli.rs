//! The one-shot subcommands: `wormholesystems <name> [args]` does its job and exits.
//! Without a subcommand the binary runs the server.
//!
//! The subcommand is the first argument and nothing else. Matching a word anywhere in the
//! arguments meant `killmails-backfill seed` would have seeded.

use sqlx::PgPool;

use crate::config::{Contact, DatabaseConfig, DiscordRegistration};
use crate::esi::EsiClient;

/// What the arguments ask for.
pub enum Command {
    Serve,
    /// Populate the reference tables from the SDE + data/static.
    Seed,
    /// Import EVE Ref's daily killmail archives for the last `days`.
    KillmailsBackfill {
        days: u32,
    },
    /// Upload the slash commands. Run it once per deploy that changes them; Discord takes a
    /// few minutes to roll a change out.
    DiscordRegister,
    /// Unpack the SDE into data/sde if it is not already there. Needs no database: CI runs
    /// it to give the parse tests their files. `--force` replaces what is there.
    SdeFetch {
        force: bool,
    },
    /// Which SDE build is loaded and whether CCP has a newer one, as `key=value` lines the
    /// deploy CLI reads.
    SdeStatus,
    /// Recompute every wormhole system's threat from the killmails already stored. The
    /// server does this daily anyway.
    ThreatAnalysis,
}

const USAGE: &str = "\
usage: wormholesystems [command]

Without a command, runs the server. Commands:
  seed                      re-seed the reference tables from the SDE
  sde-fetch [--force]       download and unpack the SDE into data/sde
  sde-status                which SDE build is loaded, and whether a newer one exists
  killmails-backfill [days] import EVE Ref's killmail archives (default 30 days)
  threat-analysis           recompute every system's threat from stored killmails
  discord-register          upload the /wh slash command to Discord";

impl Command {
    /// Parse the arguments after the program name. An unknown command is an error rather
    /// than a server start, so a typo does not boot a second server next to the real one.
    pub fn parse(args: &[String]) -> Result<Command, String> {
        let rest = args.get(1..).unwrap_or_default();
        Ok(match args.first().map(String::as_str) {
            None => Command::Serve,
            Some("seed") => Command::Seed,
            Some("killmails-backfill") => Command::KillmailsBackfill {
                days: rest.first().and_then(|d| d.parse().ok()).unwrap_or(30),
            },
            Some("discord-register") => Command::DiscordRegister,
            Some("sde-fetch") => Command::SdeFetch {
                force: rest.iter().any(|a| a == "--force"),
            },
            Some("sde-status") => Command::SdeStatus,
            Some("threat-analysis") => Command::ThreatAnalysis,
            Some("help" | "--help" | "-h") => return Err(USAGE.to_string()),
            Some(other) => return Err(format!("unknown command `{other}`\n\n{USAGE}")),
        })
    }
}

/// Run a one-shot command. [`Command::Serve`] is not one; the binary handles it.
pub async fn run(command: Command) {
    match command {
        Command::Serve => unreachable!("the server is started by the binary"),
        Command::Seed => {
            init_contact();
            crate::seed::run(&cli_db().await)
                .await
                .expect("seeding failed");
        }
        Command::KillmailsBackfill { days } => {
            init_contact();
            crate::killmails::backfill(&cli_db().await, &EsiClient::new(), days)
                .await
                .expect("backfill failed");
        }
        Command::DiscordRegister => {
            init_contact();
            let discord = DiscordRegistration::from_env().unwrap_or_else(|err| panic!("{err}"));
            match crate::discord::commands::register(&discord.application_id, &discord.bot_token)
                .await
            {
                Ok(()) => tracing::info!("registered the /wh command."),
                Err(err) => panic!("could not register commands: {err}"),
            }
        }
        Command::SdeFetch { force } => {
            init_contact();
            match tokio::task::spawn_blocking(move || crate::sde::fetch(force))
                .await
                .expect("join failed")
            {
                Ok(true) => {
                    tracing::info!("SDE downloaded and unpacked into {}", crate::sde::SDE_DIR)
                }
                Ok(false) => tracing::info!("SDE already unpacked in {}", crate::sde::SDE_DIR),
                Err(err) => panic!("could not fetch the SDE: {err}"),
            }
        }
        Command::SdeStatus => {
            init_contact();
            sde_status(&cli_db().await).await;
        }
        Command::ThreatAnalysis => {
            init_contact();
            crate::killmails::analyze(&cli_db().await, &EsiClient::new())
                .await
                .expect("threat analysis failed");
            tracing::info!("threat analysis complete.");
        }
    }
}

/// The database a subcommand works on: `.env`, `DATABASE_URL`, connected and migrated.
async fn cli_db() -> PgPool {
    let config = DatabaseConfig::from_env().unwrap_or_else(|err| panic!("{err}"));
    crate::db::connect(&config)
        .await
        .expect("db connect failed")
}

/// Every subcommand that reaches ESI, the SDE download or EVE Ref identifies itself the
/// same way the server does.
fn init_contact() {
    let contact = Contact::from_env().unwrap_or_else(|err| panic!("{err}"));
    crate::user_agent::init(&contact);
}

async fn sde_status(db: &PgPool) {
    // `query!` rather than `query_as`: the row is built from the real schema at compile
    // time, so a renamed column or a changed type stops the build instead of this.
    let loaded = sqlx::query!("select build_number, release_date from sde_build")
        .fetch_optional(db)
        .await
        .expect("could not read sde_build")
        .map(|r| (r.build_number, r.release_date));
    let latest =
        tokio::task::spawn_blocking(|| crate::sde::download::Downloader::new().latest_build())
            .await
            .expect("join failed");

    // Plain stdout, not the log: this is the command's answer, and wsctl parses it.
    match &loaded {
        Some((build, released)) => println!(
            "loaded={build}{}",
            released
                .map(|d| format!(" released={}", d.format("%Y-%m-%d")))
                .unwrap_or_default()
        ),
        None => println!("loaded=none"),
    }
    match latest {
        Ok(latest) => {
            println!("latest={}", latest.build_number);
            let stale = loaded.map(|(b, _)| b < latest.build_number).unwrap_or(true);
            println!("update_available={}", if stale { "yes" } else { "no" });
        }
        // A deploy should not fail because CCP's endpoint is having a moment.
        Err(err) => println!("latest=unknown ({err})"),
    }
}

#[cfg(test)]
mod tests {
    use super::Command;

    fn parse(args: &[&str]) -> Result<Command, String> {
        Command::parse(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn the_command_is_the_first_argument_only() {
        assert!(matches!(parse(&[]), Ok(Command::Serve)));
        assert!(matches!(parse(&["seed"]), Ok(Command::Seed)));
        assert!(matches!(
            parse(&["killmails-backfill", "seed"]),
            Ok(Command::KillmailsBackfill { days: 30 })
        ));
        assert!(matches!(
            parse(&["killmails-backfill", "7"]),
            Ok(Command::KillmailsBackfill { days: 7 })
        ));
        assert!(matches!(
            parse(&["sde-fetch", "--force"]),
            Ok(Command::SdeFetch { force: true })
        ));
        assert!(parse(&["--force", "sde-fetch"]).is_err());
        assert!(parse(&["sede"]).is_err());
    }
}
