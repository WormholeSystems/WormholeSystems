//! Set up and run WormholeSystems on one machine.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;

mod checks;
mod config;
mod envfile;
mod exec;
mod integrate;
mod rename;
mod stack;
mod ui;
mod wizard;

#[derive(Parser)]
#[command(name = "wsctl", version, about, disable_help_subcommand = true)]
struct Cli {
    /// The checkout to work in. Defaults to the current directory.
    #[arg(long, global = true)]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Check the machine, ask for what it needs, and bring the stack up
    #[command(visible_alias = "init")]
    Setup {
        /// Ask nothing: take every answer from the environment and .env, and fail naming
        /// whatever required one is missing (see docs/deployment.md)
        #[arg(long)]
        non_interactive: bool,
    },
    /// Pull, rebuild, restart, and take a newer static data export if CCP has one
    Update {
        /// Fetch the static data even when the loaded build is already current
        #[arg(long)]
        sde: bool,
        /// Skip the database dump taken before the new version starts
        #[arg(long)]
        no_backup: bool,
    },
    /// Dump the database into the backup directory now
    Backup,
    /// Replace the database with a dump, stopping the API meanwhile
    Restore {
        /// The dump, as `wsctl backup` or the scheduled backups write them
        file: PathBuf,
        /// Do not ask first
        #[arg(long)]
        yes: bool,
    },
    /// What is running, which static data is loaded, whether the URL answers
    Status,
    /// What a reverse proxy in front of the install needs to be told
    Integrate,
    /// Upload the /wh slash command to your Discord application
    DiscordRegister,
    /// Check Docker, disk, ports and DNS without changing anything
    Doctor,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("\n{} {err:#}", style("✗").red().bold());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let dir = cli.dir.unwrap_or_else(|| PathBuf::from("."));
    let runner = &mut exec::Real;

    match cli.command {
        None => {
            overview();
            Ok(())
        }
        Some(Command::Setup { non_interactive }) => {
            let vars: Vec<(String, String)> = std::env::vars().collect();
            wizard::setup(runner, &dir, non_interactive.then_some(vars.as_slice()))
        }
        Some(Command::Update { sde, no_backup }) => wizard::update(runner, &dir, sde, !no_backup),
        Some(Command::Backup) => wizard::backup(runner, &dir),
        Some(Command::Restore { file, yes }) => wizard::restore(runner, &dir, &file, yes),
        Some(Command::Status) => wizard::status(runner, &dir),
        Some(Command::Integrate) => wizard::integrate(&dir),
        Some(Command::DiscordRegister) => wizard::discord_register(runner, &dir),
        Some(Command::Doctor) => wizard::doctor(runner, &dir),
    }
}

fn overview() {
    println!(
        "{} {} — run WormholeSystems on one machine\n",
        style("wsctl").bold(),
        env!("CARGO_PKG_VERSION")
    );
    for (name, description) in [
        (
            "setup",
            "pick the database and the proxy, check the machine, start it",
        ),
        ("update", "pull, rebuild, restart, take newer static data"),
        ("backup", "dump the database now"),
        ("restore", "put a dump back"),
        ("status", "what is running, and whether it answers"),
        (
            "integrate",
            "what your own proxy needs, for each way of running one",
        ),
        ("discord-register", "upload the /wh slash command"),
        ("doctor", "check the machine and change nothing"),
    ] {
        println!("  {:<18} {description}", style(name).cyan());
    }
    println!(
        "\nStart with {}. Everything it writes lives in {} beside the compose file.",
        style("wsctl setup").cyan(),
        style(".env").cyan()
    );
}
