//! The static data the map tests read and the test database starts without: the signature
//! catalog, Thera, and wormhole systems of a given class.

use sqlx::PgPool;

/// The unidentified far side of a hole. It carries no jump mass of its own.
pub const K162: i64 = 500;
/// A hole into C3 that admits 300,000,000 kg per jump, so a medium.
pub const C247: i64 = 510;
/// A data site.
pub const RELAY: i64 = 600;
/// The hub EVE-Scout reports holes from.
pub const THERA: i64 = 31000005;

/// The signature catalog: category 1 is Wormhole with [`K162`] and [`C247`], category 2 is
/// Data with [`RELAY`], and the wormhole types behind the two holes.
pub async fn seed_catalog(pool: &PgPool) {
    for statement in [
        "insert into signature_categories (id, name, code)
         values (1, 'Wormhole', 'wormhole'), (2, 'Data Site', 'data')",
        "insert into categories (id, name) values (2, 'Celestial')",
        "insert into groups (id, category_id, name) values (988, 2, 'Wormhole')",
        "insert into types (id, group_id, name)
         values (30831, 988, 'Wormhole K162'), (30832, 988, 'Wormhole C247')",
        "insert into wormhole_types (code, type_id, max_mass_per_jump)
         values ('K162', 30831, null), ('C247', 30832, 300000000)",
        "insert into signature_types (id, signature, name, signature_category_id, target_class)
         values (500, 'K162', 'K162 - Unknown', 1, null),
                (510, 'C247', 'C247 - C3', 1, 3),
                (600, null, 'Unsecured Frontier Enclave Relay', 2, null)",
    ] {
        sqlx::query(statement).execute(pool).await.unwrap();
    }
}

/// [`THERA`], in the seeded constellation.
pub async fn seed_thera(pool: &PgPool) {
    sqlx::query(
        "insert into solar_systems (id, constellation_id, region_id, name, security_status)
         values ($1, 20000001, 10000001, 'Thera', -0.99)",
    )
    .bind(THERA)
    .execute(pool)
    .await
    .unwrap();
}

/// A J-space system of the given class in the seeded constellation, with the class on both
/// the system and its wormhole row, whichever a reader goes by.
pub async fn seed_wormhole_system(pool: &PgPool, id: i64, class: i32) {
    sqlx::query(
        "insert into solar_systems
             (id, constellation_id, region_id, name, security_status, wormhole_class_id)
         values ($1, 20000001, 10000001, $2, -1.0, $3)",
    )
    .bind(id)
    .bind(format!("J{id}"))
    .bind(class)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into wormhole_systems (solar_system_id, wormhole_class_id) values ($1, $2)",
    )
    .bind(id)
    .bind(class)
    .execute(pool)
    .await
    .unwrap();
}
