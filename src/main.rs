#[tokio::main]
async fn main() {
    wormholesystems::telemetry::init();

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
