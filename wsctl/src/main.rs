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
    Setup,
    /// Pull, rebuild, restart, and take a newer static data export if CCP has one
    Update {
        /// Fetch the static data even when the loaded build is already current
        #[arg(long)]
        sde: bool,
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
        Some(Command::Setup) => wizard::setup(runner, &dir),
        Some(Command::Update { sde }) => wizard::update(runner, &dir, sde),
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
