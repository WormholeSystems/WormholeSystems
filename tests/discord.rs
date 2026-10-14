//! The Discord half: account linking, and what unlinking takes with it.

mod common;

use sqlx::PgPool;
use wormholesystems::discord;

async fn link(pool: &PgPool, user_id: i64, discord_id: &str) {
    sqlx::query!(
        "insert into discord_accounts (user_id, discord_user_id, username)
         values ($1, $2, 'pilot')",
        user_id,
        discord_id,
    )
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn an_account_is_found_from_either_side(pool: PgPool) {
    let w = common::world(&pool).await;
    link(&pool, w.owner.user_id, "9001").await;

    let account = discord::account_for(&pool, w.owner.user_id).await.unwrap();
    assert_eq!(account.discord_user_id, "9001");
    assert_eq!(
        discord::user_for(&pool, "9001").await,
        Some(w.owner.user_id)
    );
    assert_eq!(discord::user_for(&pool, "nobody").await, None);
}

/// A direct message with nobody to send it to cannot work, so unlinking turns those alerts
/// off with a reason rather than leaving them to fail forever.
#[sqlx::test]
async fn unlinking_stops_the_alerts_that_needed_it(pool: PgPool) {
    let w = common::world(&pool).await;
    link(&pool, w.owner.user_id, "9001").await;

    let dm = sqlx::query_scalar!(
        "insert into map_alerts (map_id, created_by_user_id, name, kind, delivery, max_jumps)
         values ($1, $2, 'DM me', 'killmail', 'discord_dm', 5) returning id",
        w.map_id,
        w.owner.user_id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    // A webhook alert needs no Discord account, so it must survive.
    let destination = sqlx::query_scalar!(
        "insert into map_webhooks (map_id, name, url)
         values ($1, 'Channel', 'https://discord.com/api/webhooks/1/x') returning id",
        w.map_id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let webhook = sqlx::query_scalar!(
        "insert into map_alerts (map_id, created_by_user_id, name, kind, delivery, map_webhook_id, max_jumps)
         values ($1, $2, 'Channel', 'killmail', 'webhook', $3, 5)
         returning id",
        w.map_id,
        w.owner.user_id,
        destination,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    discord::link::unlink(&pool, w.owner.user_id).await;

    assert!(discord::account_for(&pool, w.owner.user_id).await.is_none());
    let dm_row = sqlx::query!(
        "select is_active, disabled_reason from map_alerts where id = $1",
        dm,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!dm_row.is_active);
    assert_eq!(dm_row.disabled_reason.as_deref(), Some("discord_unlinked"));

    let webhook_active =
        sqlx::query_scalar!("select is_active from map_alerts where id = $1", webhook,)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(webhook_active);

    // And it is on the record, so the settings page can explain itself.
    let events = sqlx::query_scalar!(
        "select detail from map_alert_events where map_alert_id = $1",
        dm,
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(events, vec![Some("discord_unlinked".to_string())]);
}

/// Runs `/wh` with `options`, from wherever `context` says the sender is.
async fn command(pool: &PgPool, context: &str, options: &str) -> String {
    let interaction: discord::interactions::Interaction = serde_json::from_str(&format!(
        r#"{{"type":2,"data":{{"name":"wh","options":{options},
             "resolved":{{"roles":{{"77":{{"name":"Scouts"}}}}}}}},{context}}}"#
    ))
    .unwrap();
    discord::commands::run(&common::app::state(pool), &interaction).await
}

fn in_dm(discord_id: &str) -> String {
    format!(r#""user":{{"id":"{discord_id}","username":"pilot"}}"#)
}

fn in_channel(discord_id: &str, permissions: u64) -> String {
    format!(
        r#""guild_id":"500","channel_id":"600",
           "member":{{"user":{{"id":"{discord_id}","username":"pilot"}},"permissions":"{permissions}"}}"#
    )
}

const MANAGE_CHANNELS: u64 = 1 << 4;
const MANAGE_ROLES: u64 = 1 << 28;

/// A direct message reaches nobody but its creator, so seeing the map is enough. The
/// row it leaves is the one the settings page would have written.
#[sqlx::test(migrations = "./migrations")]
async fn a_viewer_creates_a_direct_message_alert_from_discord(pool: PgPool) {
    let w = common::world(&pool).await;
    let viewer = common::member_with_role(
        &pool,
        w.owner,
        w.map_id,
        1002,
        2002,
        wormholesystems::maps::Role::Viewer,
    )
    .await;
    link(&pool, viewer.user_id, "9002").await;

    let reply = command(
        &pool,
        &in_dm("9002"),
        &format!(
            r#"[{{"name":"alert-dm","type":2,"options":[{{"name":"proximity","type":1,"options":[
                 {{"name":"map","value":"{}"}},{{"name":"system","value":"{}"}},
                 {{"name":"jumps","value":5}},{{"name":"from","value":"{}"}}]}}]}}]"#,
            w.map_id,
            common::SYS_A,
            common::SYS_B
        ),
    )
    .await;
    assert_eq!(
        reply,
        "Alert created for **Jita** within 5 jumps of **Perimeter** through the **Chain** chain."
    );

    let alerts = wormholesystems::maps::alerts::list(&pool, w.map_id)
        .await
        .unwrap();
    assert_eq!(alerts.len(), 1);
    let alert = &alerts[0];
    assert_eq!(alert.name, "Jita within 5 jumps of Perimeter");
    assert_eq!(alert.delivery.as_str(), "discord_dm");
    assert_eq!(alert.mention.as_str(), "none");
    assert_eq!(alert.target_system_name.as_deref(), Some("Jita"));
    assert_eq!(alert.origin_system_name.as_deref(), Some("Perimeter"));
    assert_eq!(alert.max_jumps, 5);
    assert!(alert.is_active);

    let creator = sqlx::query_scalar!(
        "select created_by_user_id from map_alerts where id = $1",
        alert.id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(creator, Some(viewer.user_id));
}

/// A channel post reaches a server: the map's manager decides, and Discord's own
/// permissions in that channel are respected. The role picked there is registered on the
/// map under its Discord name, so the settings page shows "Scouts", not a number.
#[sqlx::test(migrations = "./migrations")]
async fn a_channel_alert_takes_a_manager_with_the_channel_permissions(pool: PgPool) {
    let w = common::world(&pool).await;
    link(&pool, w.owner.user_id, "9001").await;
    let member = common::member_with_role(
        &pool,
        w.owner,
        w.map_id,
        1002,
        2002,
        wormholesystems::maps::Role::Member,
    )
    .await;
    link(&pool, member.user_id, "9002").await;
    let killmail = format!(
        r#"[{{"name":"alert-channel","type":2,"options":[{{"name":"killmail","type":1,"options":[
             {{"name":"map","value":"{}"}},{{"name":"jumps","value":10}},
             {{"name":"mention","value":"role"}},{{"name":"role","value":"77"}}]}}]}}]"#,
        w.map_id
    );

    assert_eq!(
        command(&pool, &in_dm("9001"), &killmail).await,
        "Channel alerts are created from inside a server channel."
    );
    let without_permission = command(&pool, &in_channel("9001", 0), &killmail).await;
    assert!(
        without_permission.contains("Manage Channels"),
        "{without_permission}"
    );
    let without_roles = command(&pool, &in_channel("9001", MANAGE_CHANNELS), &killmail).await;
    assert!(without_roles.contains("Manage Roles"), "{without_roles}");
    assert_eq!(
        command(
            &pool,
            &in_channel("9002", MANAGE_CHANNELS | MANAGE_ROLES),
            &killmail
        )
        .await,
        "Only map managers can create channel alerts."
    );
    assert!(
        wormholesystems::maps::alerts::list(&pool, w.map_id)
            .await
            .unwrap()
            .is_empty()
    );

    assert_eq!(
        command(
            &pool,
            &in_channel("9001", MANAGE_CHANNELS | MANAGE_ROLES),
            &killmail
        )
        .await,
        "Alert created for kills within 10 jumps of the **Chain** chain."
    );
    let alerts = wormholesystems::maps::alerts::list(&pool, w.map_id)
        .await
        .unwrap();
    assert_eq!(alerts.len(), 1);
    let alert = &alerts[0];
    assert_eq!(alert.name, "Kills within 10 jumps of the chain");
    assert_eq!(alert.delivery.as_str(), "discord_channel");
    assert_eq!(alert.discord_channel_id.as_deref(), Some("600"));
    assert_eq!(alert.mention.as_str(), "role");
    assert_eq!(alert.role_name.as_deref(), Some("Scouts"));
    let guild = sqlx::query_scalar!(
        "select discord_guild_id from map_alerts where id = $1",
        alert.id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(guild.as_deref(), Some("500"));
    let role = sqlx::query!(
        "select name, discord_role_id from map_webhook_roles where map_id = $1",
        w.map_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (role.name.as_str(), role.discord_role_id.as_str()),
        ("Scouts", "77")
    );
}

/// The same rules the settings page applies: a stranger gets nothing, an unknown system
/// is refused, and a role without the mention to go with it is refused before anything
/// is written.
#[sqlx::test(migrations = "./migrations")]
async fn a_discord_alert_is_refused_for_the_same_reasons_as_on_the_page(pool: PgPool) {
    let w = common::world(&pool).await;
    link(&pool, w.owner.user_id, "9001").await;
    let stranger = common::new_user(&pool).await;
    link(&pool, stranger, "9003").await;

    let jump_range = format!(
        r#"[{{"name":"alert-dm","type":2,"options":[{{"name":"jump-range","type":1,"options":[
             {{"name":"map","value":"{}"}},{{"name":"system","value":"{}"}},
             {{"name":"ship","value":"carrier"}},{{"name":"jdc","value":4}}]}}]}}]"#,
        w.map_id,
        common::SYS_A
    );
    assert_eq!(
        command(&pool, &in_dm("9003"), &jump_range).await,
        "You do not have access to that map."
    );
    let unknown_system = jump_range.replace(&common::SYS_A.to_string(), "31999999");
    assert_eq!(
        command(&pool, &in_dm("9001"), &unknown_system).await,
        "That system is unavailable."
    );
    let role_without_mention = format!(
        r#"[{{"name":"alert-channel","type":2,"options":[{{"name":"killmail","type":1,"options":[
             {{"name":"map","value":"{}"}},{{"name":"jumps","value":10}},
             {{"name":"mention","value":"none"}},{{"name":"role","value":"77"}}]}}]}}]"#,
        w.map_id
    );
    assert_eq!(
        command(
            &pool,
            &in_channel("9001", MANAGE_CHANNELS),
            &role_without_mention
        )
        .await,
        "Pick a role only when the mention is A role."
    );
    assert!(
        wormholesystems::maps::alerts::list(&pool, w.map_id)
            .await
            .unwrap()
            .is_empty()
    );

    assert_eq!(
        command(&pool, &in_dm("9001"), &jump_range).await,
        "Alert created for exits within 6.3 ly of **Jita** on **Chain**."
    );
    let alert = &wormholesystems::maps::alerts::list(&pool, w.map_id)
        .await
        .unwrap()[0];
    assert_eq!(alert.name, "Carrier range of Jita");
    assert_eq!(alert.jdc_level, Some(4));
}

/// Turning an alert off and deleting it from Discord goes through the same store as the
/// settings page: the audit trail says where it came from, and the reply what happened.
#[sqlx::test(migrations = "./migrations")]
async fn an_alert_is_switched_off_and_removed_from_discord(pool: PgPool) {
    let w = common::world(&pool).await;
    link(&pool, w.owner.user_id, "9001").await;
    let created = command(
        &pool,
        &in_dm("9001"),
        &format!(
            r#"[{{"name":"alert-dm","type":2,"options":[{{"name":"proximity","type":1,"options":[
                 {{"name":"map","value":"{}"}},{{"name":"system","value":"{}"}},
                 {{"name":"jumps","value":5}}]}}]}}]"#,
            w.map_id,
            common::SYS_A,
        ),
    )
    .await;
    assert!(created.starts_with("Alert created"), "{created}");
    let id = sqlx::query_scalar!("select id from map_alerts where map_id = $1", w.map_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let alerts_action = |action: &str| {
        format!(
            r#"[{{"name":"alerts","type":2,"options":[{{"name":"{action}","type":1,"options":[
                 {{"name":"alert","value":{id}}}]}}]}}]"#
        )
    };

    let reply = command(&pool, &in_dm("9001"), &alerts_action("disable")).await;
    assert!(reply.ends_with("is now off."), "{reply}");
    let row = sqlx::query!(
        "select is_active, disabled_reason from map_alerts where id = $1",
        id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!row.is_active);
    assert_eq!(row.disabled_reason.as_deref(), Some("manual"));
    let detail = sqlx::query_scalar!(
        "select detail from map_alert_events where map_alert_id = $1 and kind = 'disabled'",
        id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(detail.as_deref(), Some("via Discord"));

    let reply = command(&pool, &in_dm("9001"), &alerts_action("remove")).await;
    assert!(reply.starts_with("Deleted **"), "{reply}");
    let left = sqlx::query_scalar!("select count(*) from map_alerts where id = $1", id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, Some(0));
}
