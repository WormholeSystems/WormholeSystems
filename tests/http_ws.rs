//! The map socket over a real connection: it hears what commands publish, and it closes
//! once the reader's access is gone.

mod common;

use std::time::Duration;

use common::app::{app, session_cookie};
use common::{SYS_A, member_with_role, world};
use futures_util::StreamExt;
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use wormholesystems::maps::Role;
use wormholesystems::maps::access::{RevokeAccess, revoke_access};

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Serve the test app on a free port and try to open the map socket as `cookie`.
async fn connect(
    pool: &PgPool,
    map_id: i64,
    cookie: &str,
) -> Result<Socket, tokio_tungstenite::tungstenite::Error> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = app(pool);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let mut request = format!("ws://{addr}/ws/map/{map_id}")
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert("cookie", cookie.parse().unwrap());
    tokio_tungstenite::connect_async(request)
        .await
        .map(|(socket, _)| socket)
}

async fn open(pool: &PgPool, map_id: i64, cookie: &str) -> Socket {
    connect(pool, map_id, cookie).await.unwrap()
}

/// The next frame, or a failure after a few seconds rather than a hung test.
async fn next(socket: &mut Socket) -> Message {
    tokio::time::timeout(Duration::from_secs(5), socket.next())
        .await
        .expect("a frame within five seconds")
        .expect("the socket is still open")
        .unwrap()
}

#[sqlx::test]
async fn the_socket_hears_what_a_command_publishes(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let mut socket = open(&pool, w.map_id, &cookie).await;

    let placed = common::map::place(&pool, w.owner, w.map_id, SYS_A).await;

    // Other tests share the process's hub, and their maps can have this one's id in their
    // own databases; skip anything that is not this placement.
    loop {
        let Message::Text(text) = next(&mut socket).await else {
            continue;
        };
        let frame: serde_json::Value = serde_json::from_str(&text).unwrap();
        if frame["type"] == "system_added" && frame["map_solar_system_id"] == placed {
            break;
        }
    }
}

#[sqlx::test]
async fn losing_access_closes_the_socket(pool: PgPool) {
    let w = world(&pool).await;
    let member = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;
    let cookie = session_cookie(&pool, member).await;
    let mut socket = open(&pool, w.map_id, &cookie).await;

    revoke_access(
        &pool,
        w.owner,
        RevokeAccess {
            map_id: w.map_id,
            subject_id: 1002,
        },
    )
    .await
    .unwrap();

    loop {
        match next(&mut socket).await {
            Message::Close(Some(frame)) => {
                assert_eq!(u16::from(frame.code), 4403);
                break;
            }
            Message::Close(None) => panic!("closed without saying why"),
            // Frames already in flight, or another test's map with the same id.
            _ => continue,
        }
    }
}

#[sqlx::test]
async fn a_refused_socket_leaves_nothing_in_the_hub(pool: PgPool) {
    // Ids no other test's map will have, since every test shares the process's hub.
    sqlx::query("alter table maps alter column id restart with 7700000001")
        .execute(&pool)
        .await
        .unwrap();
    let w = world(&pool).await;
    assert_eq!(w.map_id, 7700000001);
    let stranger = wormholesystems::maps::Actor {
        user_id: common::new_user(&pool).await,
        character_id: 1009,
    };
    common::add_character(&pool, stranger.user_id, 1009, 2009, None).await;
    let cookie = session_cookie(&pool, stranger).await;
    let hub = wormholesystems::realtime::hub();

    // A map they may not see, and one that does not exist.
    for map_id in [w.map_id, 7700000999] {
        assert!(connect(&pool, map_id, &cookie).await.is_err());
        assert!(!hub.has_channel(map_id), "map {map_id}");
    }

    // A socket that was let in holds the channel only while it is open.
    let owner = session_cookie(&pool, w.owner).await;
    let socket = open(&pool, w.map_id, &owner).await;
    assert!(hub.has_channel(w.map_id));
    drop(socket);
    tokio::time::timeout(Duration::from_secs(5), async {
        while hub.has_channel(w.map_id) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("the channel goes once the socket closes");
}
