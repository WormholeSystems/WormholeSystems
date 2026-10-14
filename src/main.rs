#[tokio::main]
async fn main() {
    init_logging();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match wormholesystems::cli::Command::parse(&args) {
        Ok(wormholesystems::cli::Command::Serve) => {
            wormholesystems::app::serve(wormholesystems::app::config_or_exit()).await
        }
        Ok(command) => wormholesystems::cli::run(command).await,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    }
}

/// Logs go to stderr, so the few subcommands whose stdout is read by a script (`sde-status`)
/// keep it to themselves. `RUST_LOG` narrows or widens it per module; unset means `info`.
fn init_logging() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
