//! Seeds the reference tables from the SDE (`data/sde/`) and the custom static data
//! (`data/static/`). Run with `cargo run -- seed` (or the built binary + `seed`).
//!
//! Everything runs in one transaction: the entity-cycle FKs are deferred, so factions
//! and corporations (which reference each other) validate at commit. Inline FKs are
//! satisfied by insert order.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::sde::common::LocalizedString;
use crate::sde::{dogma, inventory, npc, universe};

type BoxError = Box<dyn std::error::Error>;

fn en(name: &LocalizedString) -> String {
    name.en.clone().unwrap_or_default()
}

/// Chunked multi-row insert, returning the row count.
///
/// The four-arg form ends with `on conflict do nothing`, for link tables whose primary key
/// is the whole row. The five-arg form takes an explicit conflict clause so entity tables
/// can upsert and stay in sync as the SDE changes.
macro_rules! bulk {
    ($tx:expr, $table_cols:literal, $conflict:literal, $rows:expr, |$b:ident, $r:pat_param| $body:expr) => {{
        let rows = $rows;
        for chunk in rows.chunks(1000) {
            let mut qb = QueryBuilder::<Postgres>::new(concat!("insert into ", $table_cols, " "));
            qb.push_values(chunk, |mut $b, $r| {
                $body;
            });
            qb.push(concat!(" ", $conflict));
            qb.build().execute(&mut *$tx).await?;
        }
        rows.len()
    }};
    ($tx:expr, $table_cols:literal, $rows:expr, |$b:ident, $r:pat_param| $body:expr) => {
        bulk!(
            $tx,
            $table_cols,
            "on conflict do nothing",
            $rows,
            |$b, $r| $body
        )
    };
}

/// CLI entry point (`wormholesystems seed`): always re-seeds, regardless of the loaded build.
pub async fn run() -> Result<(), BoxError> {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL")?;
    let pool = crate::db::connect(&url).await?;
    ensure_sde_present().await?;
    seed_all(&pool).await
}

/// Make sure the unpacked SDE exists under `data/sde`, downloading it on first run.
/// Blocking network and disk I/O, so it runs off the async runtime, and it logs around a
/// real download because a first boot can take 30s+.
async fn ensure_sde_present() -> Result<(), BoxError> {
    let downloaded = tokio::task::spawn_blocking(|| {
        if !std::path::Path::new(crate::sde::SDE_DIR)
            .join("_sde.jsonl")
            .exists()
        {
            tracing::info!("SDE not found under {}, downloading the latest build (~100 MB), this can take a minute…", crate::sde::SDE_DIR);
        }
        crate::sde::ensure_present()
    })
    .await??;
    if downloaded {
        tracing::info!("SDE downloaded and unpacked into {}", crate::sde::SDE_DIR);
    }
    Ok(())
}

/// Startup gate: seed only on first boot or when `data/sde` holds a newer build than
/// the one already loaded. The common case (unchanged build) is a single cheap query.
/// Returns whether a seed actually ran.
pub async fn ensure_seeded(pool: &PgPool) -> Result<bool, BoxError> {
    ensure_sde_present().await?;
    let bundled = bundled_build()?;
    // Checked against the schema at compile time, unlike the untyped form: a seed_revision
    // that changed width would otherwise only be caught by a boot that failed.
    let loaded = sqlx::query!("select build_number, seed_revision from sde_build")
        .fetch_optional(pool)
        .await?
        .map(|r| (r.build_number, r.seed_revision));
    if loaded == Some((bundled.build_number, SEED_REVISION)) {
        return Ok(false);
    }
    seed_all(pool).await?;
    Ok(true)
}

/// Bump when the seed logic or bundled static data changes in a way that requires
/// re-seeding an already-loaded SDE build.
const SEED_REVISION: i32 = 7;

/// The SDE build currently unpacked in `data/sde` (from its `_sde.jsonl` marker).
#[derive(Deserialize)]
struct SdeBuild {
    #[serde(rename = "buildNumber")]
    build_number: i64,
    #[serde(rename = "releaseDate")]
    release_date: Option<String>,
}

impl SdeBuild {
    fn release_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.release_date
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }
}

fn bundled_build() -> Result<SdeBuild, BoxError> {
    let path = std::path::Path::new(crate::sde::SDE_DIR).join("_sde.jsonl");
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(text.trim())?)
}

async fn seed_all(pool: &PgPool) -> Result<(), BoxError> {
    let categories = crate::sde::load_all::<inventory::Category>()?;
    let groups = crate::sde::load_all::<inventory::Group>()?;
    let market_groups = crate::sde::load_all::<inventory::MarketGroup>()?;
    let types = crate::sde::load_all::<inventory::Type>()?;
    let factions = crate::sde::load_all::<npc::Faction>()?;
    let corporations = crate::sde::load_all::<npc::NpcCorporation>()?;
    let regions = crate::sde::load_all::<universe::Region>()?;
    let constellations = crate::sde::load_all::<universe::Constellation>()?;
    let solar_systems = crate::sde::load_all::<universe::SolarSystem>()?;

    let mut tx = pool.begin().await?;

    let n = bulk!(
        tx,
        "categories (id, name, published)",
        "on conflict (id) do update set name = excluded.name, published = excluded.published",
        &categories,
        |b, c| {
            b.push_bind(c.id)
                .push_bind(en(&c.name))
                .push_bind(c.published)
        }
    );
    tracing::info!("categories: {n}");

    let n = bulk!(
        tx,
        "groups (id, category_id, name, published)",
        "on conflict (id) do update set category_id = excluded.category_id, name = excluded.name, published = excluded.published",
        &groups,
        |b, g| {
            b.push_bind(g.id)
                .push_bind(g.category_id)
                .push_bind(en(&g.name))
                .push_bind(g.published)
        }
    );
    tracing::info!("groups: {n}");

    let n = bulk!(
        tx,
        "market_groups (id, parent_group_id, name, has_types)",
        "on conflict (id) do update set parent_group_id = excluded.parent_group_id, name = excluded.name, has_types = excluded.has_types",
        &market_groups,
        |b, m| {
            b.push_bind(m.id)
                .push_bind(m.parent_group_id)
                .push_bind(en(&m.name))
                .push_bind(m.has_types)
        }
    );
    tracing::info!("market_groups: {n}");

    let n = bulk!(
        tx,
        "types (id, group_id, market_group_id, name, published, volume, mass, capacity, icon_id)",
        "on conflict (id) do update set group_id = excluded.group_id, market_group_id = excluded.market_group_id, name = excluded.name, published = excluded.published, volume = excluded.volume, mass = excluded.mass, capacity = excluded.capacity, icon_id = excluded.icon_id",
        &types,
        |b, t| {
            b.push_bind(t.id)
                .push_bind(t.group_id)
                .push_bind(t.market_group_id)
                .push_bind(en(&t.name))
                .push_bind(t.published)
                .push_bind(t.volume)
                .push_bind(t.mass)
                .push_bind(t.capacity)
                .push_bind(t.icon_id)
        }
    );
    tracing::info!("types: {n}");

    // factions <-> corporations form a deferred cycle; both go in this transaction.
    let n = bulk!(
        tx,
        "factions (id, name, description, corporation_id, militia_corporation_id, home_solar_system_id, size_factor)",
        "on conflict (id) do update set name = excluded.name, description = excluded.description, corporation_id = excluded.corporation_id, militia_corporation_id = excluded.militia_corporation_id, home_solar_system_id = excluded.home_solar_system_id, size_factor = excluded.size_factor",
        &factions,
        |b, f| {
            b.push_bind(f.id)
                .push_bind(en(&f.name))
                .push_bind(en(&f.description))
                .push_bind(f.corporation_id)
                .push_bind(f.militia_corporation_id)
                .push_bind(f.solar_system_id)
                .push_bind(f.size_factor)
        }
    );
    tracing::info!("factions: {n}");

    // Only the SDE-owned columns are updated: alliance_id and member_count stay ESI-managed.
    let n = bulk!(
        tx,
        "corporations (id, name, ticker, faction_id, ceo_id)",
        "on conflict (id) do update set name = excluded.name, ticker = excluded.ticker, faction_id = excluded.faction_id, ceo_id = excluded.ceo_id",
        &corporations,
        |b, c| {
            b.push_bind(c.id)
                .push_bind(en(&c.name))
                .push_bind(&c.ticker_name)
                .push_bind(c.faction_id)
                .push_bind(c.ceo_id)
        }
    );
    tracing::info!("corporations: {n}");

    let n = bulk!(
        tx,
        "regions (id, name, faction_id, wormhole_class_id)",
        "on conflict (id) do update set name = excluded.name, faction_id = excluded.faction_id, wormhole_class_id = excluded.wormhole_class_id",
        &regions,
        |b, r| {
            b.push_bind(r.id)
                .push_bind(en(&r.name))
                .push_bind(r.faction_id)
                .push_bind(r.wormhole_class_id)
        }
    );
    tracing::info!("regions: {n}");

    let n = bulk!(
        tx,
        "constellations (id, region_id, name, faction_id)",
        "on conflict (id) do update set region_id = excluded.region_id, name = excluded.name, faction_id = excluded.faction_id",
        &constellations,
        |b, c| {
            b.push_bind(c.id)
                .push_bind(c.region_id)
                .push_bind(en(&c.name))
                .push_bind(c.faction_id)
        }
    );
    tracing::info!("constellations: {n}");

    let n = bulk!(
        tx,
        "solar_systems (id, constellation_id, region_id, name, security_status, security_class, faction_id, wormhole_class_id, star_id, pos_x, pos_y, pos_z)",
        "on conflict (id) do update set constellation_id = excluded.constellation_id, region_id = excluded.region_id, name = excluded.name, security_status = excluded.security_status, security_class = excluded.security_class, faction_id = excluded.faction_id, wormhole_class_id = excluded.wormhole_class_id, star_id = excluded.star_id, pos_x = excluded.pos_x, pos_y = excluded.pos_y, pos_z = excluded.pos_z",
        &solar_systems,
        |b, s| {
            b.push_bind(s.id)
                .push_bind(s.constellation_id)
                .push_bind(s.region_id)
                .push_bind(en(&s.name))
                .push_bind(s.security_status)
                .push_bind(s.security_class.clone())
                .push_bind(s.faction_id)
                .push_bind(s.wormhole_class_id)
                .push_bind(s.star_id)
                // Metres from the galactic centre. Only jump range reads them, and only as
                // a distance between two systems, so the units never surface.
                .push_bind(s.position.x)
                .push_bind(s.position.y)
                .push_bind(s.position.z)
        }
    );
    tracing::info!("solar_systems: {n}");

    seed_entities(&mut tx, &corporations).await?;
    seed_static(&mut tx, &solar_systems).await?;

    // Effective wormhole class (see docs/database/universe.md): the SDE only sets a
    // per-system class for a handful of systems, so fall back to the wormhole-system
    // catalogue (all of J-space, Thera, C13), then to the region (Pochven, drifter
    // regions). Runs after seed_static so wormhole_systems is populated.
    sqlx::query!("update solar_systems ss
         set wormhole_class_id = coalesce(
             ss.wormhole_class_id,
             (select ws.wormhole_class_id from wormhole_systems ws where ws.solar_system_id = ss.id),
             (select r.wormhole_class_id from regions r where r.id = ss.region_id)
         )
         where ss.wormhole_class_id is null")
    .execute(&mut *tx)
    .await?;

    // A few regions carry no class in the SDE either (Syndicate is the one players can
    // reach), leaving their systems unclassified. They are ordinary known space, so the
    // security band on the CCP-rounded value settles them, the same way the client
    // classifies a system without a class.
    let unclassified = sqlx::query!(
        "select id, security_status from solar_systems where wormhole_class_id is null"
    )
    .fetch_all(&mut *tx)
    .await?;
    for row in unclassified {
        let sec = crate::util::security::ccp_round_security(row.security_status);
        let class = if sec >= 0.5 {
            7
        } else if sec >= 0.1 {
            8
        } else {
            9
        };
        sqlx::query!(
            "update solar_systems set wormhole_class_id = $1 where id = $2",
            class,
            row.id
        )
        .execute(&mut *tx)
        .await?;
    }

    // Record the loaded build + seed revision so startup can skip re-seeding when
    // nothing changed.
    let build = bundled_build()?;
    sqlx::query!(
        "insert into sde_build (id, build_number, release_date, seed_revision, loaded_at) \
         values (true, $1, $2, $3, now()) \
         on conflict (id) do update set \
         build_number = excluded.build_number, release_date = excluded.release_date, \
         seed_revision = excluded.seed_revision, loaded_at = excluded.loaded_at",
        build.build_number,
        build.release_date(),
        SEED_REVISION
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!("seeding complete (SDE build {}).", build.build_number);
    Ok(())
}

// ---- larger SDE entities: dogma catalogue + celestial topology ----
//
// Each is loaded right before its insert and dropped at the end of its scope, so the big
// celestial files (moons is ~220 MB) don't all sit in memory at once.

async fn seed_entities(
    tx: &mut sqlx::PgConnection,
    corporations: &[npc::NpcCorporation],
) -> Result<(), BoxError> {
    {
        let units = crate::sde::load_all::<dogma::DogmaUnit>()?;
        let n = bulk!(
            tx,
            "dogma_units (id, name)",
            "on conflict (id) do update set name = excluded.name",
            &units,
            |b, u| { b.push_bind(u.id).push_bind(&u.name) }
        );
        tracing::info!("dogma_units: {n}");
    }
    {
        let attrs = crate::sde::load_all::<dogma::DogmaAttribute>()?;
        let n = bulk!(
            tx,
            "dogma_attributes (id, name, unit_id, default_value, high_is_good, published)",
            "on conflict (id) do update set name = excluded.name, unit_id = excluded.unit_id, default_value = excluded.default_value, high_is_good = excluded.high_is_good, published = excluded.published",
            &attrs,
            |b, a| {
                b.push_bind(a.id)
                    .push_bind(&a.name)
                    .push_bind(a.unit_id)
                    .push_bind(a.default_value)
                    .push_bind(a.high_is_good)
                    .push_bind(a.published)
            }
        );
        tracing::info!("dogma_attributes: {n}");
    }
    {
        // typeDogma nests the attributes per type; flatten to (type_id, attribute_id, value).
        let type_dogma = crate::sde::load_all::<inventory::TypeDogma>()?;
        let mut rows = Vec::new();
        for td in &type_dogma {
            for a in &td.dogma_attributes {
                rows.push((td.id, a.attribute_id, a.value));
            }
        }
        let n = bulk!(
            tx,
            "type_attributes (type_id, attribute_id, value)",
            "on conflict (type_id, attribute_id) do update set value = excluded.value",
            &rows,
            |b, r| { b.push_bind(r.0).push_bind(r.1).push_bind(r.2) }
        );
        tracing::info!("type_attributes: {n}");
    }
    {
        let stargates = crate::sde::load_all::<universe::Stargate>()?;
        let n = bulk!(
            tx,
            "stargates (id, solar_system_id, destination_system_id, destination_stargate_id, type_id)",
            "on conflict (id) do update set solar_system_id = excluded.solar_system_id, destination_system_id = excluded.destination_system_id, destination_stargate_id = excluded.destination_stargate_id, type_id = excluded.type_id",
            &stargates,
            |b, s| {
                b.push_bind(s.id)
                    .push_bind(s.solar_system_id)
                    .push_bind(s.destination.solar_system_id)
                    .push_bind(s.destination.stargate_id)
                    .push_bind(s.type_id)
            }
        );
        tracing::info!("stargates: {n}");
    }
    {
        let planets = crate::sde::load_all::<universe::Planet>()?;
        let n = bulk!(
            tx,
            "planets (id, solar_system_id, type_id, celestial_index, name)",
            "on conflict (id) do update set solar_system_id = excluded.solar_system_id, type_id = excluded.type_id, celestial_index = excluded.celestial_index, name = excluded.name",
            &planets,
            |b, p| {
                b.push_bind(p.id)
                    .push_bind(p.solar_system_id)
                    .push_bind(p.type_id)
                    .push_bind(p.celestial_index)
                    .push_bind(p.unique_name.as_ref().map(en))
            }
        );
        tracing::info!("planets: {n}");
    }
    {
        let moons = crate::sde::load_all::<universe::Moon>()?;
        let n = bulk!(
            tx,
            "moons (id, solar_system_id, type_id, celestial_index, name)",
            "on conflict (id) do update set solar_system_id = excluded.solar_system_id, type_id = excluded.type_id, celestial_index = excluded.celestial_index, name = excluded.name",
            &moons,
            |b, m| {
                b.push_bind(m.id)
                    .push_bind(m.solar_system_id)
                    .push_bind(m.type_id)
                    .push_bind(m.celestial_index)
                    .push_bind(m.unique_name.as_ref().map(en))
            }
        );
        tracing::info!("moons: {n}");
    }
    {
        let belts = crate::sde::load_all::<universe::AsteroidBelt>()?;
        let n = bulk!(
            tx,
            "asteroid_belts (id, solar_system_id, type_id, celestial_index, name)",
            "on conflict (id) do update set solar_system_id = excluded.solar_system_id, type_id = excluded.type_id, celestial_index = excluded.celestial_index, name = excluded.name",
            &belts,
            |b, a| {
                b.push_bind(a.id)
                    .push_bind(a.solar_system_id)
                    .push_bind(a.type_id)
                    .push_bind(a.celestial_index)
                    .push_bind(a.unique_name.as_ref().map(en))
            }
        );
        tracing::info!("asteroid_belts: {n}");
    }
    {
        // Station owners are NPC corps; null any owner we didn't seed so the
        // (deferred) corporations FK still validates at commit. The SDE carries no
        // station names, so they are synthesized the way the game builds them:
        // "{System} {Planet} - Moon {N} - {Corp} {Operation}".
        let corp_ids: HashSet<i64> = corporations.iter().map(|c| c.id).collect();
        let corp_names: HashMap<i64, String> = corporations
            .iter()
            .filter_map(|c| Some((c.id, c.name.en.clone()?)))
            .collect();
        let operations = crate::sde::load_all::<npc::StationOperation>()?;
        let op_names: HashMap<i64, String> = operations
            .iter()
            .filter_map(|o| Some((o.id, o.operation_name.en.clone()?)))
            .collect();
        let system_names: HashMap<i64, String> = sqlx::query!("select id, name from solar_systems")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|r| (r.id, r.name))
            .collect();

        fn roman(n: i32) -> String {
            const NUMERALS: [(i32, &str); 13] = [
                (1000, "M"),
                (900, "CM"),
                (500, "D"),
                (400, "CD"),
                (100, "C"),
                (90, "XC"),
                (50, "L"),
                (40, "XL"),
                (10, "X"),
                (9, "IX"),
                (5, "V"),
                (4, "IV"),
                (1, "I"),
            ];
            let mut n = n;
            let mut out = String::new();
            for (value, numeral) in NUMERALS {
                while n >= value {
                    out.push_str(numeral);
                    n -= value;
                }
            }
            out
        }

        let stations = crate::sde::load_all::<npc::NpcStation>()?;
        let n = bulk!(
            tx,
            "stations (id, solar_system_id, type_id, owner_corporation_id, operation_id, name)",
            "on conflict (id) do update set solar_system_id = excluded.solar_system_id, type_id = excluded.type_id, owner_corporation_id = excluded.owner_corporation_id, operation_id = excluded.operation_id, name = excluded.name",
            &stations,
            |b, s| {
                let owner = Some(s.owner_id).filter(|id| corp_ids.contains(id));
                let mut name = system_names
                    .get(&{ s.solar_system_id })
                    .cloned()
                    .unwrap_or_default();
                if let Some(planet) = s.celestial_index.filter(|p| *p > 0) {
                    name.push(' ');
                    name.push_str(&roman(planet));
                }
                if let Some(moon) = s.orbit_index.filter(|m| *m > 0) {
                    name.push_str(&format!(" - Moon {moon}"));
                }
                if let Some(corp) = corp_names.get(&{ s.owner_id }) {
                    name.push_str(" - ");
                    name.push_str(corp);
                }
                if s.use_operation_name
                    && let Some(op) = op_names.get(&{ s.operation_id })
                {
                    name.push(' ');
                    name.push_str(op);
                }
                b.push_bind(s.id)
                    .push_bind(s.solar_system_id)
                    .push_bind(s.type_id)
                    .push_bind(owner)
                    .push_bind(s.operation_id)
                    .push_bind(Some(name))
            }
        );
        tracing::info!("stations: {n}");
    }
    {
        // Station services and the per-operation service sets, for the navigation
        // Find ("nearest repair / cloning / ..." lookups).
        let services = crate::sde::load_all::<npc::StationService>()?;
        let n = bulk!(
            tx,
            "station_services (id, name)",
            "on conflict (id) do update set name = excluded.name",
            &services,
            |b, s| {
                b.push_bind(s.id)
                    .push_bind(s.service_name.en.clone().unwrap_or_default())
            }
        );
        tracing::info!("station_services: {n}");

        let operations = crate::sde::load_all::<npc::StationOperation>()?;
        let service_ids: HashSet<i64> = services.iter().map(|s| s.id).collect();
        let pairs: Vec<(i64, i64)> = operations
            .iter()
            .flat_map(|op| {
                op.services
                    .iter()
                    .map(|svc| (op.id, *svc as i64))
                    .filter(|(_, svc)| service_ids.contains(svc))
                    .collect::<Vec<_>>()
            })
            .collect();
        let n = bulk!(
            tx,
            "station_operation_services (operation_id, service_id)",
            "on conflict (operation_id, service_id) do nothing",
            &pairs,
            |b, (op, svc)| { b.push_bind(op).push_bind(svc) }
        );
        tracing::info!("station_operation_services: {n}");
    }
    Ok(())
}

// ---- custom static data (data/static/*.json) ----

#[derive(Deserialize)]
struct WormholesFile {
    wormholes: HashMap<String, WhType>,
}
#[derive(Deserialize)]
struct WhType {
    #[serde(rename = "typeID")]
    type_id: i64,
    dest: Option<i32>,
    #[serde(default)]
    src: Vec<i32>,
    #[serde(rename = "static")]
    is_static: Option<bool>,
    max_mass_per_jump: Option<i64>,
    total_mass: Option<i64>,
    mass_regen: Option<i64>,
    lifetime: Option<f64>,
    sibling_groups: Option<serde_json::Value>,
    signature_strength: Option<f64>,
}

#[derive(Deserialize)]
struct WhSystemsFile {
    systems: Vec<WhSystem>,
}
#[derive(Deserialize)]
struct WhSystem {
    id: i64,
    class: Option<i32>,
    effect: Option<String>,
    #[serde(default)]
    statics: Vec<String>,
    #[serde(default)]
    shattered: bool,
}

#[derive(Deserialize)]
struct SignaturesFile {
    categories: Vec<SigCategory>,
    types: Vec<SigType>,
}
#[derive(Deserialize)]
struct SigCategory {
    id: i64,
    name: String,
    code: String,
}
#[derive(Deserialize)]
struct SigType {
    id: i64,
    name: String,
    signature: Option<String>,
    signature_category_id: i64,
    target_class: Option<i32>,
    extra: Option<String>,
    #[serde(default)]
    spawn_areas: Vec<i32>,
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, BoxError> {
    Ok(serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open(path)?,
    ))?)
}

async fn seed_static(
    tx: &mut sqlx::PgConnection,
    solar_systems: &[universe::SolarSystem],
) -> Result<(), BoxError> {
    // wormhole effects + modifiers
    let effects: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> =
        read_json("data/static/wormhole_effects.json")?;
    let effect_names: Vec<String> = effects.keys().cloned().collect();
    let mut modifiers = Vec::new();
    for (effect, kinds) in &effects {
        for (kind_name, stats) in kinds {
            let kind = if kind_name == "Buffs" {
                "buff"
            } else {
                "debuff"
            };
            for (stat, values) in stats {
                for (i, value) in values.iter().enumerate() {
                    modifiers.push((
                        effect.clone(),
                        kind,
                        stat.clone(),
                        (i + 1) as i32,
                        value.clone(),
                    ));
                }
            }
        }
    }
    let n = bulk!(tx, "wormhole_effects (name)", &effect_names, |b, name| {
        b.push_bind(name)
    });
    tracing::info!("wormhole_effects: {n}");
    let n = bulk!(
        tx,
        "wormhole_effect_modifiers (effect_name, kind, stat, wormhole_class_id, value)",
        &modifiers,
        |b, m| {
            b.push_bind(&m.0)
                .push_bind(m.1)
                .push_bind(&m.2)
                .push_bind(m.3)
                .push_bind(&m.4)
        }
    );
    tracing::info!("wormhole_effect_modifiers: {n}");

    // wormhole types + sources
    let wh: WormholesFile = read_json("data/static/wormholes.json")?;
    let wh_types: Vec<(String, WhType)> = wh.wormholes.into_iter().collect();
    let mut sources = Vec::new();
    for (code, t) in &wh_types {
        for src in &t.src {
            sources.push((code.clone(), *src));
        }
    }
    let n = bulk!(
        tx,
        "wormhole_types (code, type_id, dest_class, is_static, max_mass_per_jump, total_mass, mass_regen, lifetime_hours, sibling_groups, signature_strength)",
        "on conflict (code) do update set type_id = excluded.type_id, dest_class = excluded.dest_class, is_static = excluded.is_static, max_mass_per_jump = excluded.max_mass_per_jump, total_mass = excluded.total_mass, mass_regen = excluded.mass_regen, lifetime_hours = excluded.lifetime_hours, sibling_groups = excluded.sibling_groups, signature_strength = excluded.signature_strength",
        &wh_types,
        |b, (code, t)| {
            b.push_bind(code)
                .push_bind(t.type_id)
                .push_bind(t.dest)
                .push_bind(t.is_static)
                .push_bind(t.max_mass_per_jump)
                .push_bind(t.total_mass)
                .push_bind(t.mass_regen)
                .push_bind(t.lifetime)
                .push_bind(t.sibling_groups.clone())
                .push_bind(t.signature_strength)
        }
    );
    tracing::info!("wormhole_types: {n}");
    let n = bulk!(
        tx,
        "wormhole_type_sources (wormhole_code, wormhole_class_id)",
        &sources,
        |b, s| { b.push_bind(&s.0).push_bind(s.1) }
    );
    tracing::info!("wormhole_type_sources: {n}");

    // wormhole systems + statics (need a real class; skip rows without one)
    let whs: WhSystemsFile = read_json("data/static/wormhole_systems.json")?;
    let wh_systems: Vec<WhSystem> = whs
        .systems
        .into_iter()
        .filter(|s| s.class.is_some())
        .collect();
    let mut statics = Vec::new();
    for s in &wh_systems {
        for code in &s.statics {
            statics.push((s.id, code.clone()));
        }
    }
    let n = bulk!(
        tx,
        "wormhole_systems (solar_system_id, wormhole_class_id, effect_name, is_shattered)",
        "on conflict (solar_system_id) do update set wormhole_class_id = excluded.wormhole_class_id, effect_name = excluded.effect_name, is_shattered = excluded.is_shattered",
        &wh_systems,
        |b, s| {
            b.push_bind(s.id)
                .push_bind(s.class.unwrap())
                .push_bind(s.effect.clone())
                .push_bind(s.shattered)
        }
    );
    tracing::info!("wormhole_systems: {n}");
    let n = bulk!(
        tx,
        "wormhole_system_statics (solar_system_id, wormhole_code)",
        &statics,
        |b, s| { b.push_bind(s.0).push_bind(&s.1) }
    );
    tracing::info!("wormhole_system_statics: {n}");

    // signature catalogue
    let sig: SignaturesFile = read_json("data/static/signatures.json")?;
    let mut spawn_areas = Vec::new();
    for t in &sig.types {
        for area in &t.spawn_areas {
            spawn_areas.push((t.id, *area));
        }
    }
    let n = bulk!(
        tx,
        "signature_categories (id, name, code)",
        &sig.categories,
        |b, c| { b.push_bind(c.id).push_bind(&c.name).push_bind(&c.code) }
    );
    tracing::info!("signature_categories: {n}");
    let n = bulk!(
        tx,
        "signature_types (id, signature, name, signature_category_id, target_class, extra)",
        &sig.types,
        |b, t| {
            b.push_bind(t.id)
                .push_bind(&t.signature)
                .push_bind(&t.name)
                .push_bind(t.signature_category_id)
                .push_bind(t.target_class)
                .push_bind(t.extra.clone())
        }
    );
    tracing::info!("signature_types: {n}");
    let n = bulk!(
        tx,
        "signature_type_spawn_areas (signature_type_id, wormhole_class_id)",
        &spawn_areas,
        |b, s| { b.push_bind(s.0).push_bind(s.1) }
    );
    tracing::info!("signature_type_spawn_areas: {n}");

    // jove observatories: source is region -> [system names]; resolve names to ids.
    let by_name: HashMap<String, i64> = solar_systems.iter().map(|s| (en(&s.name), s.id)).collect();
    let jove: HashMap<String, Vec<String>> = read_json("data/static/jove_observatories.json")?;
    let mut jove_ids: Vec<i64> = jove
        .values()
        .flatten()
        .filter_map(|name| by_name.get(name).copied())
        .collect();
    jove_ids.sort_unstable();
    jove_ids.dedup();
    let n = bulk!(
        tx,
        "jove_observatories (solar_system_id)",
        &jove_ids,
        |b, id| { b.push_bind(*id) }
    );
    tracing::info!("jove_observatories: {n}");

    Ok(())
}
