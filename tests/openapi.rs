//! The OpenAPI document: it is written to `docs/openapi.json` here (the same way `cargo
//! test` regenerates the TypeScript bindings), it is served, and nothing reaches the
//! router without a path attribute that puts it in the document.

mod common;

use axum::http::StatusCode;
use common::app::{app, get};
use sqlx::PgPool;
use std::path::Path;

fn source_files() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/api");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, std::fs::read_to_string(&path).unwrap())
        })
        .collect();
    files.sort();
    files
}

/// Regenerates the committed document. CI checks the result is what is in git, so a
/// changed handler cannot ship without its description.
#[test]
fn the_document_is_written_to_docs() {
    let api = wormholesystems::api::openapi();
    let json = api.to_pretty_json().unwrap() + "\n";
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/openapi.json");
    std::fs::write(&path, json).unwrap();

    let parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(parsed["openapi"], "3.1.0");
    assert_eq!(parsed["info"]["title"], "WormholeSystems API");
    assert_eq!(parsed["info"]["version"], env!("CARGO_PKG_VERSION"));
    let paths = parsed["paths"].as_object().unwrap();
    assert!(paths.len() > 90, "{} paths", paths.len());
    assert!(paths.contains_key("/api/maps/{id}/systems/add"));
    assert!(!paths.keys().any(|path| path.contains("/v1/")));
    for (path, item) in paths {
        for (method, operation) in item.as_object().unwrap() {
            assert!(
                operation["responses"]
                    .as_object()
                    .is_some_and(|r| !r.is_empty()),
                "{method} {path} says nothing about its responses"
            );
            assert!(
                operation["tags"].as_array().is_some_and(|t| !t.is_empty()),
                "{method} {path} has no tag"
            );
            // Spelled out per operation even where it is the default, for the clients
            // that never look at the top-level one.
            assert!(
                operation["security"]
                    .as_array()
                    .is_some_and(|s| !s.is_empty()),
                "{method} {path} says nothing about who may call it"
            );
        }
    }
    let schemes = parsed["components"]["securitySchemes"].as_object().unwrap();
    assert_eq!(schemes["bearer"]["scheme"], "bearer");
    assert_eq!(schemes["session"]["in"], "cookie");
}

/// The only way onto the router is `routes!()`, which refuses a handler without a path
/// attribute. This keeps it that way: a plain `.route()` in an area module would bypass
/// the document, and a handler without the attribute would be one `routes!()` cannot take.
#[test]
fn every_handler_is_in_the_document() {
    for (name, source) in source_files() {
        if name == "mod.rs" || name == "ws.rs" {
            continue;
        }
        assert!(
            !source.contains(".route("),
            "{name} registers a route outside the document; use .routes(routes!(..))"
        );
        let handlers = source.matches("): State<AppState>").count();
        let documented = source.matches("#[utoipa::path(").count();
        assert_eq!(
            handlers, documented,
            "{name}: {handlers} handlers take State but {documented} carry #[utoipa::path]"
        );
    }
}

#[sqlx::test]
async fn the_document_and_its_viewer_are_served_without_signing_in(pool: PgPool) {
    let spec = get(app(&pool), "/api/openapi.json", None).await;
    assert_eq!(spec.status, StatusCode::OK);
    assert_eq!(spec.body["info"]["title"], "WormholeSystems API");
    assert!(spec.body["paths"]["/api/maps"].is_object());

    let response = app(&pool)
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/docs")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(content_type.starts_with("text/html"), "{content_type}");
}

use tower::ServiceExt;
