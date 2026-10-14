//! The JSON HTTP API: plain Axum handlers over the [`crate::maps`] actions, plus the
//! realtime WebSocket handlers. This is the client/server boundary for the SvelteKit
//! frontend.
//!
//! One module per area, each owning its handlers, its routes and the wire types it serves.
//! [`extract`] holds the shared request plumbing, [`router`] merges the area routers.
//!
//! The acting [`Actor`](crate::maps::Actor) is resolved server-side from the session
//! cookie or a bearer token ([`extract::Credentials`]), never sent by the client. Each
//! mutating handler publishes the matching [`MapEvent`](crate::maps::MapEvent) to the hub
//! after the action commits.
pub mod access;
pub mod alerts;
pub mod background_image;
pub mod connections;
pub mod eve_scout;
pub mod extract;
pub mod history;
pub mod identity;
pub mod ignored;
pub mod killmails;
pub mod layout;
pub mod maps;
pub mod reference;
pub mod search;
pub mod signatures;
pub mod systems;
pub mod tokens;
pub mod tracking;
pub mod transfer;
pub mod user_settings;
pub mod watchlist;
pub mod ws;

use axum::Json;
use axum::Router;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Serialize;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi, ToResponse, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::auth::AppState;
use crate::maps::MapError;

// Wire types live next to their handlers; these re-exports save the rest of the crate from
// caring which area a type belongs to.
pub use access::AccessSubject;
pub use connections::ShipSearchResult;
pub use eve_scout::EveScoutConnection;
pub use identity::{CharacterRef, CharacterStatus, CharacterSummary, ScopeStatus};
pub use layout::{BreakpointLayout, LayoutItem, PANEL_IDS, PanelLayouts, validate_layouts};
pub use maps::{MapCharacter, MapEntry};
pub use reference::{SystemSearchResult, ThreatAnalysis, ThreatEntity};
pub use search::{MapSearchHit, ThreatMatch};
pub use signatures::{SignatureCatalog, SignatureCategoryInfo, SignatureTypeInfo};
pub use user_settings::{MapUserSettings, UpdateMapUserSettings};

/// An API error: a status code plus a message, rendered as `{"error": "..."}`.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

/// What every error answers with.
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

// The error responses, named once so every handler can point at the same description.
// They exist for the document only; the field is what says the body is an `ErrorBody`.
#[allow(dead_code)]
#[derive(ToResponse)]
#[response(
    description = "The request does not make sense: a missing field, a value out of range, or a body naming a different map than the URL"
)]
pub struct BadRequest(ErrorBody);

#[allow(dead_code)]
#[derive(ToResponse)]
#[response(description = "Nobody is signed in, or the bearer token is unknown, revoked or expired")]
pub struct Unauthorized(ErrorBody);

#[allow(dead_code)]
#[derive(ToResponse)]
#[response(description = "Signed in, but the role on this map does not allow it")]
pub struct Forbidden(ErrorBody);

#[allow(dead_code)]
#[derive(ToResponse)]
#[response(description = "No such thing, or not one the caller may know about")]
pub struct NotFound(ErrorBody);

#[allow(dead_code)]
#[derive(ToResponse)]
#[response(description = "The change contradicts the map's current state")]
pub struct Conflict(ErrorBody);

#[allow(dead_code)]
#[derive(ToResponse)]
#[response(description = "EVE's API refused or failed")]
pub struct UpstreamFailed(ErrorBody);

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    pub fn unauthorized() -> Self {
        ApiError {
            status: StatusCode::UNAUTHORIZED,
            message: "not authenticated".into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}

impl From<MapError> for ApiError {
    fn from(err: MapError) -> Self {
        let status = match &err {
            MapError::NotFound => StatusCode::NOT_FOUND,
            MapError::Forbidden => StatusCode::FORBIDDEN,
            MapError::Conflict(_) | MapError::LastOwner => StatusCode::CONFLICT,
            MapError::Validation(_) => StatusCode::BAD_REQUEST,
            MapError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        // The status already says which kind of failure it is, so the message is just the
        // part a person can act on: the form shows it as is, and "invalid: " in front of it
        // would only be noise.
        let message = match err {
            MapError::Conflict(message) | MapError::Validation(message) => message,
            MapError::Db(err) => return ApiError::from(err),
            other => other.to_string(),
        };
        ApiError { status, message }
    }
}

impl From<sqlx::Error> for ApiError {
    /// A database error says things about the schema and the query that the caller has no
    /// use for and no business seeing. It goes to the log, and the response says only that
    /// it was on our side.
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("database error: {err}");
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "something went wrong on our side".into(),
        }
    }
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;

const DESCRIPTION: &str = "\
Everything the map screen does goes over this API, and all of it is open to a script \
holding a personal access token. It is generated from the same handlers and wire types \
the frontend uses, so it is complete and current by construction.

**Pre-alpha.** The API follows the frontend and changes with it, without notice and \
without a migration path. Pin the version you tested against and expect to revisit.

Sign in with the session cookie a browser holds, or send `Authorization: Bearer wst_...` \
with a token minted under Settings, API tokens. A token acts as the user who made it, with \
exactly their access. Endpoints marked with no security requirement are public; the ones \
that also accept a share token (`?share=`) say so in their parameters.

Live updates go over WebSockets, which this document cannot describe: see \
`docs/realtime-api.md` in the repository.";

#[derive(OpenApi)]
#[openapi(
    info(
        title = "WormholeSystems API",
        description = DESCRIPTION,
        license(name = "MIT", url = "https://github.com/WormholeSystems/wormholesystems-next/blob/main/LICENSE"),
    ),
    modifiers(&Security),
    security(("bearer" = []), ("session" = [])),
    tags(
        (name = "maps", description = "The maps themselves: list yours, create one, read a whole map in one call, rename or reconfigure it, and who is on it right now. Everything else hangs off a map id from here."),
        (name = "access", description = "Who may see and edit a map. Roles are granted to a character, corporation or alliance, ownership can be handed over, and a share token gives read-only access without an account."),
        (name = "systems", description = "The nodes on the map: placing and removing solar systems, moving them, and everything written on a node such as its alias, status, notes, occupier, and the home and rally markers. Also the unresolved hole placeholders and clearing a map."),
        (name = "connections", description = "The wormholes between two placed systems: adding and removing them, setting mass and lifetime status, the jump log that feeds the mass estimate, and pruning connections nobody has used in a while."),
        (name = "signatures", description = "Scan results per system: pasting the probe scanner, editing single signatures, and linking a wormhole signature to the connection it turned out to be. The type catalogue for the pickers lives here too."),
        (name = "tracking", description = "Jump tracking on behalf of a pilot: telling the map a character moved from one system to another, so the hole gets mapped and linked the way the automatic tracker would."),
        (name = "history", description = "The map's undo history: the event log, stepping back and forward, and jumping to a point in it. Every write elsewhere lands here as an event."),
        (name = "watchlist", description = "Systems a map keeps an eye on for routing: the list, adding and removing entries, and pinning the ones that matter most."),
        (name = "ignored systems", description = "Systems the whole map routes around and never auto-places, kept on the server so route planning, Discord routes and alerts all agree. Manager only."),
        (name = "eve scout", description = "The public holes out of Thera and Turnur as EVE Scout's scouts report them, and putting one hub's connections on the map in one step."),
        (name = "alerts", description = "What a map watches for and where it tells people: proximity, killmail and jump-range alerts, their delivery to Discord channels or DMs, the roles that can be mentioned, and the webhooks a map posts to."),
        (name = "killmails", description = "Recent kills in and around the chain, as the map's killmail feed shows them."),
        (name = "search", description = "Searching within one map: systems, aliases, signatures and notes in one query."),
        (name = "transfer", description = "Moving a map between instances: export to a file, import into an existing map, or create a new map from a file. The format is the one the legacy app reads and writes."),
        (name = "user settings", description = "One person's preferences on one map: routing tolerances, what the tracker does for them, which panels are shown and where. Nothing here affects other viewers."),
        (name = "background image", description = "A picture behind one viewer's map: upload, fetch and remove, with the layout mode set through user settings."),
        (name = "identity", description = "The signed-in user: their characters, which one they act as, ESI scopes, Discord link, and setting in-game waypoints through ESI."),
        (name = "tokens", description = "Personal access tokens for scripts. Managed only from a browser session; the plaintext is shown once at creation."),
        (name = "reference", description = "Static and instance-wide data the frontend needs: the universe graph for routing, wormhole effects, signature types, threat analysis, skyhook timers, server status and the instance's configuration."),
    ),
    components(
        responses(BadRequest, Unauthorized, Forbidden, NotFound, Conflict, UpstreamFailed),
        // The WebSocket frames, so a client generated from the document has them too.
        schemas(crate::maps::MapEvent, crate::user_channel::UserEvent),
    ),
)]
struct ApiDoc;

/// The two ways in: the bearer token a script sends, and the cookie a browser holds.
struct Security;

impl Modify for Security {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("wst_...")
                    .description(Some("A personal access token from Settings, API tokens"))
                    .build(),
            ),
        );
        components.add_security_scheme(
            "session",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                crate::session::SESSION_COOKIE,
                "The session a browser holds after signing in with EVE",
            ))),
        );
    }
}

/// Every area's routes on one router, carrying the document they describe.
fn areas() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(identity::routes())
        .merge(reference::routes())
        .merge(eve_scout::routes())
        .merge(maps::routes())
        .merge(systems::routes())
        .merge(connections::routes())
        .merge(signatures::routes())
        .merge(watchlist::routes())
        .merge(ignored::routes())
        .merge(search::routes())
        .merge(access::routes())
        .merge(history::routes())
        .merge(killmails::routes())
        .merge(tracking::routes())
        .merge(user_settings::routes())
        .merge(background_image::routes())
        .merge(alerts::routes())
        .merge(transfer::routes())
        .merge(tokens::routes())
}

/// The OpenAPI document for everything [`router`] serves.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let (_, api) = areas().split_for_parts();
    api
}

/// The JSON API, compressed for clients that ask for gzip or zstd, so the standalone setup
/// gets it without Caddy in front. The websockets are routed outside this, and the layer's
/// default predicate leaves images and replies too small to bother with alone.
pub fn router() -> Router<AppState> {
    let (router, api) = areas().split_for_parts();
    let spec = api.clone();
    router
        .route("/api/openapi.json", get(move || async move { Json(spec) }))
        .merge(Scalar::with_url("/api/docs", api))
        .layer(tower_http::compression::CompressionLayer::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refused_change_answers_with_its_message_alone() {
        let invalid = ApiError::from(MapError::Validation("pick a system".into()));
        assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
        assert_eq!(invalid.message, "pick a system");

        let conflict = ApiError::from(MapError::Conflict("already on the map".into()));
        assert_eq!(conflict.status, StatusCode::CONFLICT);
        assert_eq!(conflict.message, "already on the map");
    }

    #[test]
    fn a_database_error_does_not_reach_the_caller() {
        let leaky = || sqlx::Error::Protocol("relation \"secret_table\" does not exist".into());
        for err in [
            ApiError::from(leaky()),
            ApiError::from(MapError::Db(leaky())),
        ] {
            assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
            assert!(!err.message.contains("secret_table"), "{}", err.message);
        }
    }
}
