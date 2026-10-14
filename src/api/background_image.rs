//! The picture behind a viewer's own map: uploading one, fetching it back, taking it
//! away. The layout mode is an ordinary field of the user settings; only the file itself
//! lives here, because a file does not travel well inside JSON.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::{ApiError, ApiResult};
use crate::app::AppState;
use crate::maps::Role;
use crate::maps::background::MAX_IMAGE_BYTES;
use crate::maps::user_settings::{self, MapUserSettings};

/// Room for the multipart framing around an image at the size limit; the image itself is
/// checked against the real limit once it is out of the envelope.
const UPLOAD_BODY_LIMIT: usize = MAX_IMAGE_BYTES + 64 * 1024;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(show, upload, remove))
        .layer(DefaultBodyLimit::max(UPLOAD_BODY_LIMIT))
}

/// `GET /api/maps/{id}/background-image`: the caller's own image on this map. The `?v=`
/// the settings URL carries is only a cache key; the row says which file it is.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/background-image",
    tag = "background image",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, description = "The image bytes", content_type = "image/*"), super::MapReadErrors),
)]
async fn show(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> Result<Response, ApiError> {
    let actor = super::extract::require_role_on_map(&state, &creds, map_id, Role::Viewer).await?;
    let stored = state
        .backgrounds
        .image_for(&state.db, map_id, actor.user_id)
        .await?;
    let Some((format, bytes)) = stored else {
        return Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: "no background image".into(),
        });
    };
    Ok((
        [
            (header::CONTENT_TYPE, format.content_type()),
            // Immutable: a replacement is a different URL, so this one never changes.
            (
                header::CACHE_CONTROL,
                "private, max-age=31536000, immutable",
            ),
        ],
        bytes,
    )
        .into_response())
}

/// The one part the upload carries. Documentation only: the handler reads the multipart
/// stream itself.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
struct BackgroundImageUpload {
    /// PNG, JPEG, GIF or WebP, at most 8 MiB.
    #[schema(value_type = String, format = Binary, content_media_type = "image/*")]
    image: Vec<u8>,
}

/// `PUT /api/maps/{id}/background-image`: multipart with one `image` part. Replaces the
/// previous image and answers with the caller's settings, URL included.
#[utoipa::path(
    put,
    path = "/api/maps/{id}/background-image",
    tag = "background image",
    params(("id" = i64, Path, description = "The map")),
    request_body(content = BackgroundImageUpload, content_type = "multipart/form-data"),
    responses((status = 200, body = MapUserSettings, description = "OK"), super::MapWriteErrors),
)]
async fn upload(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    mut multipart: Multipart,
) -> ApiResult<MapUserSettings> {
    let actor = super::extract::require_role_on_map(&state, &creds, map_id, Role::Viewer).await?;

    let mut image = None;
    while let Some(field) = multipart.next_field().await.map_err(multipart_error)? {
        if field.name() == Some("image") {
            image = Some(field.bytes().await.map_err(multipart_error)?);
            break;
        }
    }
    let Some(bytes) = image else {
        return Err(ApiError::bad_request(
            "send the picture as the `image` part",
        ));
    };

    state
        .backgrounds
        .replace(&state.db, map_id, actor.user_id, &bytes)
        .await?;
    Ok(Json(
        user_settings::load(&state.db, map_id, actor.user_id).await?,
    ))
}

/// `DELETE /api/maps/{id}/background-image`: back to the plain grid.
#[utoipa::path(
    delete,
    path = "/api/maps/{id}/background-image",
    tag = "background image",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = MapUserSettings, description = "OK"), super::MapReadErrors),
)]
async fn remove(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<MapUserSettings> {
    let actor = super::extract::require_role_on_map(&state, &creds, map_id, Role::Viewer).await?;
    state
        .backgrounds
        .clear(&state.db, map_id, actor.user_id)
        .await?;
    Ok(Json(
        user_settings::load(&state.db, map_id, actor.user_id).await?,
    ))
}

fn multipart_error(err: axum::extract::multipart::MultipartError) -> ApiError {
    ApiError::bad_request(format!("could not read the upload: {err}"))
}
