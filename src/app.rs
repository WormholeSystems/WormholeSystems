//! The server process: the state every handler shares, the router, and the startup that
//! builds both and the background jobs around them.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::config::{Config, ConfigError};
use crate::esi::{EsiClient, Sso};

/// What every handler can reach. Cheap to clone: everything in it is a handle.
///
/// The map event hub and the canvas geometry are not here. Both are process globals
/// ([`crate::maps::hub`], [`crate::maps::grid`]) because a command, which has only its
/// transaction, publishes and places through them; a second copy here would be one that
/// could disagree.
#[derive(Clone)]
pub struct AppState {
    pub sso: Arc<Sso>,
    pub esi: EsiClient,
    pub db: PgPool,
    pub user_hub: crate::realtime::UserHub,
    /// The latest Tranquility status, for the header indicator.
    pub server: crate::server_status::ServerWatch,
    /// `None` when no Discord application is configured, which disables the bot half.
    pub discord: Option<Arc<crate::config::DiscordConfig>>,
    /// Whether this deployment is served over HTTPS, taken from the callback URL EVE was
    /// given. The session cookie is marked `Secure` when it is, so a browser will not send
    /// it in the clear; local development over plain http would never receive it back.
    pub secure_cookies: bool,
    /// Where the per-viewer map background images live.
    pub backgrounds: crate::maps::BackgroundStore,
    /// Who may sign in. Checked at the SSO callback only, so it decides who gets a session
    /// and whose characters join an account; a session already open runs out as usual.
    pub allowed_logins: crate::allowlist::AffiliationAllowlist,
    /// Who may create maps, independent of who may log in: an instance can admit a whole
    /// alliance while only its leadership creates maps.
    pub map_creators: crate::allowlist::AffiliationAllowlist,
    /// The site-wide banner, if the operator set one.
    pub announcement: Option<Arc<crate::announcement::Announcement>>,
    /// EVE Scout's public Thera and Turnur holes, with the last read cached.
    pub eve_scout: Arc<crate::integrations::eve_scout::EveScout>,
}

/// Everything the server answers: sign-in, Discord, the realtime sockets, the JSON API and
/// the health check.
pub fn router(state: AppState) -> Router {
    Router::new()
        // Outside the API router, so it is not compressed, not in the OpenAPI document,
        // and answers without a session.
        .route("/healthz", get(healthz))
        .route("/auth/login", get(crate::auth::login))
        .route("/auth/callback", get(crate::auth::callback))
        .route("/auth/logout", get(crate::auth::logout))
        // Discord: linking an account, and the bot's interaction endpoint.
        .route("/discord/connect", get(crate::discord::link::connect))
        .route("/discord/callback", get(crate::discord::link::callback))
        .route(
            "/discord/interactions",
            post(crate::discord::interactions::handle),
        )
        // Realtime: per-map event stream + the per-user private channel / activity heartbeat.
        .route("/ws/map/{map_id}", get(crate::api::ws::map_ws))
        .route("/ws/user", get(crate::api::ws::user_ws))
        // The JSON API for the SvelteKit frontend.
        .merge(crate::api::router())
        .with_state(state)
}

/// Start the server from its configuration and run it until a shutdown signal.
///
/// Config, the database and SSO are all essential, so this fails fast, with a message that
/// says what to fix, if any is unavailable.
pub async fn serve(config: Config) {
    crate::user_agent::init(&config.contact);
    let db = crate::db::connect(&config.database).await.expect(
        "could not connect to Postgres. Is it running? start it with `docker compose up -d`",
    );
    // Keep the reference tables in sync with the bundled SDE. Runs only on first boot
    // or when data/sde holds a newer build; otherwise it's a single cheap query.
    match crate::seed::ensure_seeded(&db).await {
        Ok(true) => tracing::info!("seeded reference tables from the SDE"),
        Ok(false) => {}
        Err(e) => panic!("could not seed the SDE reference tables: {e}"),
    }
    // Read before the config is consumed: the callback URL is what says whether anyone
    // reaches this over HTTPS, which decides whether the session cookie may be `Secure`.
    let secure_cookies = config.sso.redirect_uri.starts_with("https://");
    let sso = Arc::new(
        Sso::discover(crate::user_agent::client(), config.sso.clone())
            .await
            .expect("could not reach the EVE SSO: check your network connection"),
    );
    if config.esi_base_url != crate::esi::BASE_URL {
        tracing::info!("ESI: {}", config.esi_base_url);
    }
    let esi = EsiClient::with_config(
        crate::user_agent::client(),
        &config.esi_base_url,
        crate::esi::COMPATIBILITY_DATE,
    );
    // Commands place nodes of their own (a paste raising ghosts), and a command has only
    // its transaction, so the geometry is installed for the process here.
    crate::maps::set_grid(config.grid);

    let user_hub = crate::realtime::UserHub::new();
    let server = start_background(&config, &db, &sso, &esi, &user_hub).await;

    let state = AppState {
        sso,
        esi,
        db,
        user_hub,
        server,
        discord: config.discord.clone().map(Arc::new),
        secure_cookies,
        backgrounds: crate::maps::BackgroundStore::new(config.uploads_dir),
        allowed_logins: config.allowed_logins,
        map_creators: config.map_creators,
        announcement: config.announcement.map(Arc::new),
        eve_scout: Arc::new(crate::integrations::eve_scout::EveScout::new(
            config.eve_scout_url,
        )),
    };

    let addr = config.listen_addr;
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| panic!("could not listen on {addr}: {err}"));
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, router(state).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server failed");
    // Requests have drained; let a sweep that is mid-pass finish, inside the time `docker
    // stop` allows before it kills the process.
    crate::background::shutdown(Duration::from_secs(5)).await;
    tracing::info!("stopped");
}

/// Read the configuration, or stop with a message that says what is wrong with it.
pub fn config_or_exit() -> Config {
    Config::from_env().unwrap_or_else(|err| match err {
        ConfigError::Missing(_) => {
            panic!("{err}: copy .env.example to .env and fill in the variables")
        }
        ConfigError::Invalid(..) => panic!("invalid configuration, {err}"),
    })
}

/// Every background job, started in the order they depend on each other. Returns the
/// server watch, which the jobs gate on and the API serves.
async fn start_background(
    config: &Config,
    db: &PgPool,
    sso: &Arc<Sso>,
    esi: &EsiClient,
    user_hub: &crate::realtime::UserHub,
) -> crate::server_status::ServerWatch {
    // Is Tranquility up? Everything below that talks to ESI gates on it, so it is started
    // first.
    let server = crate::server_status::start(
        db.clone(),
        esi.clone(),
        user_hub.clone(),
        config.polling.server_status,
    );

    // Poll live character status for active users, pinging each user's private channel on
    // a change.
    crate::pilots::poller::start(
        db.clone(),
        sso.clone(),
        esi.clone(),
        user_hub.clone(),
        server.clone(),
    );

    // Keep every account character's corp/alliance current, since corp and alliance
    // grants on a map match against them.
    crate::affiliation::start(db.clone(), esi.clone(), server.clone());

    // Keep sovereignty (and its alliance/corp entities) current for map display.
    crate::sovereignty::start(db.clone(), esi.clone(), server.clone());

    // Mirror the raidable skyhooks CCP is currently advertising.
    crate::skyhooks::start(
        db.clone(),
        esi.clone(),
        server.clone(),
        config.polling.skyhooks,
    );
    crate::maps::access::spawn_expiry_sweep(db.clone());

    // Discord alerts. One runtime shared by everything that can fire one, since the
    // stargate graph it holds is the expensive part.
    let bot_token = config.discord.as_ref().and_then(|d| d.bot_token.clone());
    let alerts = match crate::alerts::Runtime::load(db, bot_token).await {
        Ok(runtime) => Some(Arc::new(runtime)),
        Err(err) => {
            tracing::error!("alerts disabled, could not load the stargate graph: {err}");
            None
        }
    };
    if let Some(alerts) = alerts.clone() {
        crate::alerts::start(db.clone(), alerts);
    }

    // Killmail ingest + daily threat analysis (gated by ZKB_LISTEN=1).
    crate::killmails::start(db.clone(), esi.clone(), alerts, &config.killmails);

    // Purge stale signatures (legacy expiry: 3d wormholes, 7d sites).
    crate::maps::signatures::start_expiry(db.clone());

    // Prune unclaimed connection-jump observations.
    crate::maps::jumps::start_prune(db.clone());

    // Age wormhole connections to EOL / critical and remove the dead ones.
    crate::maps::connection::start_lifecycle(db.clone());

    // Drop command-journal entries past the undo retention window.
    crate::maps::history::start_purge(db.clone());

    // Drop old alert deliveries and audit rows, and expired sessions.
    crate::retention::start(db.clone());

    server
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
    crate::background::begin_shutdown();
}

/// `GET /healthz`: up, and the database answers. What Docker's healthcheck asks, so it is
/// cheap, needs no session and says nothing beyond a word.
///
/// The query has a deadline of its own: a pool with every connection taken would otherwise
/// hold this for the pool's acquire timeout, longer than the healthcheck waits.
pub async fn healthz(State(state): State<AppState>) -> (StatusCode, &'static str) {
    let ping = sqlx::query_scalar!("select 1 as \"one!\"").fetch_one(&state.db);
    match tokio::time::timeout(Duration::from_secs(3), ping).await {
        Ok(Ok(_)) => (StatusCode::OK, "ok"),
        Ok(Err(err)) => {
            tracing::warn!("health check: database error: {err}");
            (StatusCode::SERVICE_UNAVAILABLE, "database unavailable")
        }
        Err(_) => {
            tracing::warn!("health check: database did not answer in time");
            (StatusCode::SERVICE_UNAVAILABLE, "database unavailable")
        }
    }
}
