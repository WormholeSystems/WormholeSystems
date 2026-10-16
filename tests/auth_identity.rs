//! The SSO login → identity resolution rules from authentication.md: account reuse,
//! character-transfer (owner-hash change) reassignment, linking, preferred selection, and
//! the session lifecycle. These are pure DB logic, so they're driven directly with a pool.

use sqlx::PgPool;
use wormholesystems::esi::jwt::Claims;
use wormholesystems::session::{
    Entity, SESSION_DAYS, actor_for_session, create_session, delete_session,
    ensure_preferred_character, persist_identity, set_active_character, set_preferred_character,
};

fn claims(character_id: i64, owner_hash: &str) -> Claims {
    Claims {
        character_id,
        name: format!("Pilot {character_id}"),
        owner_hash: owner_hash.into(),
        scopes: vec![],
    }
}

fn corp(id: i64) -> Entity {
    Entity {
        id,
        name: format!("Corp {id}"),
        ticker: "CRP".into(),
    }
}

async fn character_user(pool: &PgPool, character_id: i64) -> i64 {
    sqlx::query_scalar("select user_id from characters where id = $1")
        .bind(character_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn user_exists(pool: &PgPool, user_id: i64) -> bool {
    sqlx::query_scalar!("select exists(select 1 from users where id = $1)", user_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(false)
}

async fn is_preferred(pool: &PgPool, character_id: i64) -> bool {
    sqlx::query_scalar("select is_preferred from characters where id = $1")
        .bind(character_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn new_character_creates_a_preferred_character_and_caches_its_corp(pool: PgPool) {
    let user = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();

    assert_eq!(character_user(&pool, 100).await, user);
    assert!(
        is_preferred(&pool, 100).await,
        "first character is preferred"
    );

    // The corp entity was cached (deferred FK target).
    let corp_count: i64 = sqlx::query_scalar("select count(*) from corporations where id = 2001")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(corp_count, 1);
}

#[sqlx::test]
async fn returning_login_reuses_the_account_and_refreshes_corp(pool: PgPool) {
    let user = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    // Same owner hash, different corp → same user, corp updated.
    let again = persist_identity(&pool, &claims(100, "h1"), corp(2002), None, None)
        .await
        .unwrap();

    assert_eq!(user, again, "returning login keeps the same account");
    let (uid, corp_id): (i64, i64) =
        sqlx::query_as("select user_id, corporation_id from characters where id = 100")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(uid, user);
    assert_eq!(corp_id, 2002, "corp refreshed on login");
    let users: i64 = sqlx::query_scalar("select count(*) from users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 1, "no extra account created");
}

#[sqlx::test]
async fn owner_hash_change_reassigns_the_character_to_a_new_account(pool: PgPool) {
    let old = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    // The character was transferred (new owner hash) → must land on a fresh account.
    let new = persist_identity(&pool, &claims(100, "h2"), corp(2001), None, None)
        .await
        .unwrap();

    assert_ne!(old, new, "transfer must not sign into the previous account");
    assert_eq!(character_user(&pool, 100).await, new);
    let owner: String = sqlx::query_scalar("select owner_hash from characters where id = 100")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(owner, "h2");
    assert!(
        is_preferred(&pool, 100).await,
        "first character of the new account is preferred"
    );
}

#[sqlx::test]
async fn linking_attaches_a_character_without_making_it_preferred(pool: PgPool) {
    let user = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    // A second character linked to the signed-in user.
    let linked = persist_identity(&pool, &claims(200, "h2"), corp(2001), None, Some(user))
        .await
        .unwrap();

    assert_eq!(linked, user, "linked character joins the existing account");
    assert_eq!(character_user(&pool, 200).await, user);
    assert!(
        is_preferred(&pool, 100).await,
        "the first character stays preferred"
    );
    assert!(
        !is_preferred(&pool, 200).await,
        "a linked second character is not preferred"
    );
}

/// Both people signed up separately, then one hands their character to the other. Each
/// account had a preferred character of its own, and carrying that flag across gave the
/// receiving account two, which the unique index refuses: the link failed outright.
#[sqlx::test]
async fn linking_a_character_that_already_has_its_own_account(pool: PgPool) {
    let first = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    let second = persist_identity(&pool, &claims(200, "h2"), corp(2001), None, None)
        .await
        .unwrap();
    assert_ne!(first, second, "two separate accounts to begin with");
    assert!(
        is_preferred(&pool, 200).await,
        "each is preferred in its own"
    );

    let linked = persist_identity(&pool, &claims(200, "h2"), corp(2001), None, Some(first))
        .await
        .unwrap();

    assert_eq!(linked, first, "the character joins the other account");
    assert_eq!(character_user(&pool, 200).await, first);
    assert!(
        is_preferred(&pool, 100).await,
        "the receiving account keeps the character it preferred"
    );
    assert!(
        !is_preferred(&pool, 200).await,
        "the arriving character does not bring its old account's preference with it"
    );
    assert!(
        !user_exists(&pool, second).await,
        "the account it left has no characters, so nothing can sign into it again"
    );
}

/// The emptied account goes, but not one that still has somebody in it.
#[sqlx::test]
async fn an_account_with_characters_left_is_not_removed(pool: PgPool) {
    let first = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    let second = persist_identity(&pool, &claims(200, "h2"), corp(2001), None, None)
        .await
        .unwrap();
    persist_identity(&pool, &claims(300, "h3"), corp(2001), None, Some(second))
        .await
        .unwrap();

    persist_identity(&pool, &claims(200, "h2"), corp(2001), None, Some(first))
        .await
        .unwrap();

    assert!(
        user_exists(&pool, second).await,
        "character 300 is still there, so the account stays"
    );
    assert!(
        is_preferred(&pool, 300).await,
        "and it takes over as that account's preferred one"
    );
}

/// Days left on a session, rounded.
async fn days_left(pool: &PgPool, session: &str) -> i64 {
    sqlx::query_scalar::<_, f64>(
        "select extract(epoch from expires_at - now())::float8 / 86400 from sessions where id = $1",
    )
    .bind(session)
    .fetch_one(pool)
    .await
    .unwrap()
    .round() as i64
}

async fn set_days_left(pool: &PgPool, session: &str, days: i64) {
    sqlx::query(
        "update sessions set expires_at = now() + make_interval(days => $2::int) where id = $1",
    )
    .bind(session)
    .bind(days as i32)
    .execute(pool)
    .await
    .unwrap();
}

/// Staying signed in: a session lasts as long as a browser keeps a cookie, and using it
/// starts that again. One that already ran out stays out.
#[sqlx::test]
async fn a_session_in_use_keeps_renewing(pool: PgPool) {
    let user = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    let session = create_session(&pool, user, 100).await.unwrap();
    assert_eq!(days_left(&pool, &session).await, SESSION_DAYS);

    set_days_left(&pool, &session, 3).await;
    assert!(actor_for_session(&pool, &session).await.unwrap().is_some());
    assert_eq!(
        days_left(&pool, &session).await,
        SESSION_DAYS,
        "renewed by use"
    );

    set_days_left(&pool, &session, -1).await;
    assert!(actor_for_session(&pool, &session).await.unwrap().is_none());
    assert_eq!(
        days_left(&pool, &session).await,
        -1,
        "an expired one is not revived"
    );
}

#[sqlx::test]
async fn session_lifecycle_and_character_switch(pool: PgPool) {
    let user = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    persist_identity(&pool, &claims(200, "h2"), corp(2001), None, Some(user))
        .await
        .unwrap();
    // A character on a *different* account, to test the switch guard.
    let other = persist_identity(&pool, &claims(300, "h3"), corp(2001), None, None)
        .await
        .unwrap();
    assert_ne!(other, user);

    let session = create_session(&pool, user, 100).await.unwrap();
    let actor = actor_for_session(&pool, &session).await.unwrap().unwrap();
    assert_eq!((actor.user_id, actor.character_id), (user, 100));

    // Switch to another of the user's own characters → allowed.
    assert!(set_active_character(&pool, &session, 200).await.unwrap());
    let actor = actor_for_session(&pool, &session).await.unwrap().unwrap();
    assert_eq!(actor.character_id, 200);

    // Switching to a character the user doesn't own → rejected, unchanged.
    assert!(!set_active_character(&pool, &session, 300).await.unwrap());
    assert_eq!(
        actor_for_session(&pool, &session)
            .await
            .unwrap()
            .unwrap()
            .character_id,
        200
    );

    // Logout invalidates the session.
    delete_session(&pool, &session).await.unwrap();
    assert!(actor_for_session(&pool, &session).await.unwrap().is_none());
}

#[sqlx::test]
async fn a_new_session_starts_as_the_preferred_character(pool: PgPool) {
    let user = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    persist_identity(&pool, &claims(200, "h2"), corp(2001), None, Some(user))
        .await
        .unwrap();

    // Signing in as the alt still lands on the preferred character: which one the SSO
    // handed back is not a choice the user made.
    let session = create_session(&pool, user, 200).await.unwrap();
    let actor = actor_for_session(&pool, &session).await.unwrap().unwrap();
    assert_eq!(actor.character_id, 100);

    // Preferring the alt moves the default; the character switcher is per session and is
    // left alone.
    assert!(set_preferred_character(&pool, user, 200).await.unwrap());
    assert!(is_preferred(&pool, 200).await);
    assert!(!is_preferred(&pool, 100).await, "at most one per user");
    assert_eq!(
        actor_for_session(&pool, &session)
            .await
            .unwrap()
            .unwrap()
            .character_id,
        100,
        "an open session keeps acting as whoever it was"
    );

    let next = create_session(&pool, user, 100).await.unwrap();
    assert_eq!(
        actor_for_session(&pool, &next)
            .await
            .unwrap()
            .unwrap()
            .character_id,
        200
    );
}

#[sqlx::test]
async fn preferring_a_character_of_another_account_is_refused(pool: PgPool) {
    let user = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    persist_identity(&pool, &claims(300, "h3"), corp(2001), None, None)
        .await
        .unwrap();

    assert!(!set_preferred_character(&pool, user, 300).await.unwrap());
    assert!(
        is_preferred(&pool, 100).await,
        "the refused write leaves the old preference alone"
    );
}

#[sqlx::test]
async fn an_account_left_without_a_preference_gets_one_back(pool: PgPool) {
    let user = persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    persist_identity(&pool, &claims(200, "h2"), corp(2001), None, Some(user))
        .await
        .unwrap();

    // Removing the preferred character is what leaves the gap.
    sqlx::query("delete from characters where id = 100")
        .execute(&pool)
        .await
        .unwrap();
    ensure_preferred_character(&pool, user).await.unwrap();
    assert!(is_preferred(&pool, 200).await);

    // And it does not steal the flag from an account that already has one.
    persist_identity(&pool, &claims(400, "h4"), corp(2001), None, Some(user))
        .await
        .unwrap();
    ensure_preferred_character(&pool, user).await.unwrap();
    assert!(is_preferred(&pool, 200).await);
    assert!(!is_preferred(&pool, 400).await);
}

#[sqlx::test]
async fn a_saved_token_replaces_the_old_one_and_keeps_its_scopes(pool: PgPool) {
    use std::time::{Duration, SystemTime};
    use wormholesystems::db::PgTokenStore;
    use wormholesystems::esi::{Token, TokenStore};

    persist_identity(&pool, &claims(100, "h1"), corp(2001), None, None)
        .await
        .unwrap();
    let store = PgTokenStore::new(pool.clone());
    let token = |access: &str, scopes: &[&str]| Token {
        access_token: access.into(),
        refresh_token: "refresh".into(),
        expires_at: SystemTime::now() + Duration::from_secs(1200),
        scopes: scopes.iter().map(|s| s.to_string()).collect(),
    };

    store.save(100, &token("first", &["a", "b"])).await.unwrap();
    // A scope named twice, one already known and one new: each stored once.
    store
        .save(100, &token("second", &["b", "c", "c"]))
        .await
        .unwrap();

    let loaded = store.load(100).await.unwrap().unwrap();
    assert_eq!(loaded.access_token, "second");
    let mut scopes = loaded.scopes;
    scopes.sort();
    assert_eq!(scopes, ["b", "c"]);
    let tokens: i64 = sqlx::query_scalar("select count(*) from esi_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tokens, 1);
}
