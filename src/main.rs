#[tokio::main]
async fn main() {
    init_logging();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match wormholesystems::cli::Command::parse(&args) {
        Ok(wormholesystems::cli::Command::Serve) => {}
        Ok(command) => return wormholesystems::cli::run(command).await,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    }

    use std::sync::Arc;

    use axum::Router;
    use axum::routing::get;
    use wormholesystems::auth::{self, AppState, Auth};
    use wormholesystems::config::{Config, ConfigError};
    use wormholesystems::esi::{EsiClient, Sso};

    // Config, the database, and SSO are all essential, so fail fast if any is unavailable.
    let config = Config::from_env().unwrap_or_else(|err| match err {
        ConfigError::Missing(_) => {
            panic!("{err}: copy .env.example to .env and fill in the variables")
        }
        ConfigError::Invalid(..) => panic!("invalid configuration, {err}"),
    });
    wormholesystems::user_agent::init(&config.contact);
    let db = wormholesystems::db::connect(&config.database).await.expect(
        "could not connect to Postgres. Is it running? start it with `docker compose up -d`",
    );
    // Keep the reference tables in sync with the bundled SDE. Runs only on first boot
    // or when data/sde holds a newer build; otherwise it's a single cheap query.
    match wormholesystems::seed::ensure_seeded(&db).await {
        Ok(true) => tracing::info!("seeded reference tables from the SDE"),
        Ok(false) => {}
        Err(e) => panic!("could not seed the SDE reference tables: {e}"),
    }
    // Read before the config is consumed: the callback URL is what says whether anyone
    // reaches this over HTTPS, which decides whether the session cookie may be `Secure`.
    let secure_cookies = config.sso.redirect_uri.starts_with("https://");
    let sso = Arc::new(
        Sso::discover(wormholesystems::user_agent::client(), config.sso)
            .await
            .expect("could not reach the EVE SSO: check your network connection"),
    );
    if config.esi_base_url != wormholesystems::esi::BASE_URL {
        tracing::info!("ESI: {}", config.esi_base_url);
    }
    let esi = EsiClient::with_config(
        wormholesystems::user_agent::client(),
        &config.esi_base_url,
        wormholesystems::esi::COMPATIBILITY_DATE,
    );
    let auth = Arc::new(Auth::new(sso.clone(), esi.clone()));

    let hub = wormholesystems::maps::hub().clone();
    let user_hub = wormholesystems::user_channel::UserHub::new();

    // Background: is Tranquility up? Everything below that talks to ESI gates on it, so it
    // is started first.
    let server = wormholesystems::server_status::start(
        db.clone(),
        esi.clone(),
        user_hub.clone(),
        config.polling.server_status,
    );

    // Background: poll live character status for active users, pinging each user's private
    // channel on a change.
    wormholesystems::tracking::start(
        db.clone(),
        sso.clone(),
        esi.clone(),
        user_hub.clone(),
        hub.clone(),
        server.clone(),
    );

    // Background: keep every account character's corp/alliance current, since corp and
    // alliance grants on a map match against them.
    wormholesystems::affiliation::start(db.clone(), esi.clone(), server.clone());

    // Background: keep sovereignty (and its alliance/corp entities) current for map display.
    wormholesystems::sovereignty::start(db.clone(), esi.clone(), server.clone());

    // Background: mirror the raidable skyhooks CCP is currently advertising.
    wormholesystems::skyhooks::start(
        db.clone(),
        esi.clone(),
        server.clone(),
        config.polling.skyhooks,
    );
    wormholesystems::maps::access::spawn_expiry_sweep(db.clone());

    // Background: Discord alerts. One runtime shared by everything that can fire one, since
    // the stargate graph it holds is the expensive part.
    let bot_token = config.discord.as_ref().and_then(|d| d.bot_token.clone());
    let alerts = match wormholesystems::alerts::Runtime::load(&db, bot_token).await {
        Ok(runtime) => Some(Arc::new(runtime)),
        Err(err) => {
            tracing::error!("alerts disabled, could not load the stargate graph: {err}");
            None
        }
    };
    if let Some(alerts) = alerts.clone() {
        wormholesystems::alerts::start(db.clone(), hub.clone(), alerts);
    }

    // Background: killmail ingest + daily threat analysis (gated by ZKB_LISTEN=1).
    wormholesystems::killmails::start(
        db.clone(),
        esi.clone(),
        hub.clone(),
        alerts,
        &config.killmails,
    );

    // Background: purge stale signatures (legacy expiry: 3d wormholes, 7d sites).
    wormholesystems::maps::signatures::start_expiry(db.clone(), hub.clone());

    // Background: prune unclaimed connection-jump observations.
    wormholesystems::maps::jumps::start_prune(db.clone());

    // Background: age wormhole connections to EOL / critical and remove the dead ones.
    wormholesystems::maps::connection::start_lifecycle(db.clone());

    // Background: drop command-journal entries past the undo retention window.
    wormholesystems::maps::events_log::start_purge(db.clone());

    // Background: drop old alert deliveries and audit rows, and expired sessions.
    wormholesystems::retention::start(db.clone());

    // Commands place nodes of their own (a paste raising ghosts), and a command has only
    // its transaction, so the geometry is installed for the process here.
    wormholesystems::maps::set_grid(config.grid);

    let state = AppState {
        auth,
        discord: config.discord.clone().map(Arc::new),
        db,
        hub,
        user_hub,
        grid: config.grid,
        server,
        secure_cookies,
        backgrounds: wormholesystems::maps::BackgroundStore::new(config.uploads_dir),
        allowed_logins: config.allowed_logins,
        map_creators: config.map_creators,
        announcement: config.announcement.map(Arc::new),
        eve_scout_url: config.eve_scout_url.into(),
    };

    let app = Router::new()
        // Outside the API router, so it is not compressed, not in the OpenAPI document,
        // and answers without a session.
        .route("/healthz", get(wormholesystems::app::healthz))
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/logout", get(auth::logout))
        // Discord: linking an account, and the bot's interaction endpoint.
        .route(
            "/discord/connect",
            get(wormholesystems::discord::link::connect),
        )
        .route(
            "/discord/callback",
            get(wormholesystems::discord::link::callback),
        )
        .route(
            "/discord/interactions",
            axum::routing::post(wormholesystems::discord::interactions::handle),
        )
        // Realtime: per-map event stream + the per-user private channel / activity heartbeat.
        .route("/ws/map/{map_id}", get(wormholesystems::api::ws::map_ws))
        .route("/ws/user", get(wormholesystems::api::ws::user_ws))
        // The JSON API for the SvelteKit frontend.
        .merge(wormholesystems::api::router())
        .with_state(state);

    let addr = config.listen_addr;
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    // Requests have drained; let a sweep that is mid-pass finish, inside the time `docker
    // stop` allows before it kills the process.
    wormholesystems::background::shutdown(std::time::Duration::from_secs(5)).await;
    tracing::info!("stopped");
}

/// SIGTERM is what `docker stop` sends, Ctrl-C (SIGINT) what a terminal does. Either one
/// stops accepting connections, closes the open WebSockets and lets in-flight requests
/// finish, rather than waiting out Docker's ten seconds and being killed.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
    tracing::info!("shutting down");
    // WebSockets outlive the request that opened them, so the server would wait on them
    // forever; they watch this and close themselves.
    wormholesystems::background::begin_shutdown();
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
