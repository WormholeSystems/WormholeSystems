#[tokio::main]
async fn main() {
    // `wormholesystems seed` populates the reference tables from the SDE + data/static, then exits.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "seed") {
        wormholesystems::seed::run().await.expect("seeding failed");
        return;
    }
    // `wormholesystems killmails-backfill [days]` imports EVE Ref daily archives, then exits.
    if let Some(pos) = args.iter().position(|a| a == "killmails-backfill") {
        let days: u32 = args.get(pos + 1).and_then(|d| d.parse().ok()).unwrap_or(30);
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let db = wormholesystems::db::connect(&url)
            .await
            .expect("db connect failed");
        wormholesystems::killmails::backfill(&db, &wormholesystems::esi::EsiClient::new(), days)
            .await
            .expect("backfill failed");
        return;
    }

    // `wormholesystems discord-register` uploads the slash commands, then exits. Run it once per
    // deploy that changes them; Discord takes a few minutes to roll a change out.
    if args.iter().any(|a| a == "discord-register") {
        dotenvy::dotenv().ok();
        let application_id =
            std::env::var("DISCORD_APPLICATION_ID").expect("DISCORD_APPLICATION_ID not set");
        let token = std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN not set");
        match wormholesystems::discord::commands::register(&application_id, &token).await {
            Ok(()) => println!("registered the /wh command."),
            Err(err) => panic!("could not register commands: {err}"),
        }
        return;
    }

    // `wormholesystems sde-fetch` unpacks the SDE into data/sde if it is not already there, then
    // exits. Needs no database: CI runs it to give the parse tests their files.
    // `--force` replaces what is there with the current build.
    if args.iter().any(|a| a == "sde-fetch") {
        dotenvy::dotenv().ok();
        let force = args.iter().any(|a| a == "--force");
        match tokio::task::spawn_blocking(move || wormholesystems::sde::fetch(force))
            .await
            .expect("join failed")
        {
            Ok(true) => println!(
                "SDE downloaded and unpacked into {}",
                wormholesystems::sde::SDE_DIR
            ),
            Ok(false) => println!("SDE already unpacked in {}", wormholesystems::sde::SDE_DIR),
            Err(err) => panic!("could not fetch the SDE: {err}"),
        }
        return;
    }

    // `wormholesystems sde-status` says which SDE build is loaded and whether CCP has a newer one.
    // The deploy CLI reads this to decide whether an update is worth pulling.
    if args.iter().any(|a| a == "sde-status") {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let db = wormholesystems::db::connect(&url)
            .await
            .expect("db connect failed");
        // `query!` rather than `query_as`: the row is built from the real schema at compile
        // time, so a renamed column or a changed type stops the build instead of this.
        let loaded = sqlx::query!("select build_number, release_date from sde_build")
            .fetch_optional(&db)
            .await
            .expect("could not read sde_build")
            .map(|r| (r.build_number, r.release_date));
        let latest = tokio::task::spawn_blocking(|| {
            wormholesystems::sde::download::Downloader::new().latest_build()
        })
        .await
        .expect("join failed");

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
        return;
    }

    // `wormholesystems threat-analysis` recomputes every wormhole system's threat from the killmails
    // already stored, then exits. The server does this daily anyway.
    if args.iter().any(|a| a == "threat-analysis") {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let db = wormholesystems::db::connect(&url)
            .await
            .expect("db connect failed");
        wormholesystems::killmails::analyze(&db, &wormholesystems::esi::EsiClient::new())
            .await
            .expect("threat analysis failed");
        println!("threat analysis complete.");
        return;
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
    let db = wormholesystems::db::connect(&config.database_url)
        .await
        .expect(
            "could not connect to Postgres. Is it running? start it with `docker compose up -d`",
        );
    // Keep the reference tables in sync with the bundled SDE. Runs only on first boot
    // or when data/sde holds a newer build; otherwise it's a single cheap query.
    match wormholesystems::seed::ensure_seeded(&db).await {
        Ok(true) => println!("seeded reference tables from the SDE"),
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
        println!("ESI: {}", config.esi_base_url);
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
    let server = wormholesystems::server_status::start(db.clone(), esi.clone(), user_hub.clone());

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

    // Background: keep sovereignty (and its alliance/corp entities) current for map display.
    wormholesystems::sovereignty::start(db.clone(), esi.clone(), server.clone());

    // Background: mirror the raidable skyhooks CCP is currently advertising.
    wormholesystems::skyhooks::start(db.clone(), esi.clone(), server.clone());
    wormholesystems::maps::access::spawn_expiry_sweep(db.clone());

    // Background: Discord alerts. One runtime shared by everything that can fire one, since
    // the stargate graph it holds is the expensive part.
    let bot_token = config.discord.as_ref().and_then(|d| d.bot_token.clone());
    let alerts = match wormholesystems::alerts::Runtime::load(&db, bot_token).await {
        Ok(runtime) => Some(Arc::new(runtime)),
        Err(err) => {
            eprintln!("alerts disabled, could not load the stargate graph: {err}");
            None
        }
    };
    if let Some(alerts) = alerts.clone() {
        wormholesystems::alerts::start(db.clone(), hub.clone(), alerts);
    }

    // Background: killmail ingest + daily threat analysis (gated by ZKB_LISTEN=1).
    wormholesystems::killmails::start(db.clone(), esi.clone(), hub.clone(), alerts);

    // Background: purge stale signatures (legacy expiry: 3d wormholes, 7d sites).
    wormholesystems::maps::signatures::start_expiry(db.clone(), hub.clone());

    // Background: prune unclaimed connection-jump observations.
    wormholesystems::maps::jumps::start_prune(db.clone());

    // Background: age wormhole connections to EOL / critical and remove the dead ones.
    wormholesystems::maps::connection::start_lifecycle(db.clone());

    // Background: drop command-journal entries past the undo retention window.
    wormholesystems::maps::events_log::start_purge(db.clone());

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
    };

    let app = Router::new()
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

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
