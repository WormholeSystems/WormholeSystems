//! `GET /api/instance`: what the deployment has switched on, and the operator's banner.

mod common;

use std::sync::Arc;

use axum::http::StatusCode;
use common::app::{get, state};
use sqlx::PgPool;
use wormholesystems::announcement::{Announcement, AnnouncementVars};

#[sqlx::test]
async fn no_announcement_is_null(pool: PgPool) {
    let res = get(common::app::app(&pool), "/api/instance", None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.body["announcement"].is_null());
}

// Anonymous visitors get it too: the landing page shows it.
#[sqlx::test]
async fn the_announcement_is_served_to_everyone(pool: PgPool) {
    let mut state = state(&pool);
    state.announcement = Announcement::from_vars(AnnouncementVars {
        level: Some("warning"),
        title: Some("Moving to a new address"),
        message: Some("This page moves on October 1st."),
        link_url: Some("https://example.com"),
        link_label: Some("Read more"),
        dismissible: Some("false"),
    })
    .map(Arc::new);
    let app = wormholesystems::api::router().with_state(state);

    let res = get(app, "/api/instance", None).await;
    let a = &res.body["announcement"];
    assert_eq!(a["level"], "warning");
    assert_eq!(a["title"], "Moving to a new address");
    assert_eq!(a["message"], "This page moves on October 1st.");
    assert_eq!(a["link"]["url"], "https://example.com");
    assert_eq!(a["link"]["label"], "Read more");
    assert_eq!(a["dismissible"], false);
    assert_eq!(a["id"].as_str().map(str::len), Some(12));
}
