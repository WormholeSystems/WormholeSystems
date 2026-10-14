//! The slash commands.
//!
//! One `/wh` with subcommands, so WormholeSystems claims a single name in a server's command
//! list. Every reply is ephemeral and every command needs a linked account, because
//! everything they answer is about the sender's own maps.

use serde_json::{Value, json};

use crate::alerts::ships::JumpShip;
use crate::alerts::{AlertDelivery, AlertKind, AlertMention, filters};
use crate::auth::AppState;
use crate::maps::alerts::SaveAlert;
use crate::maps::{MapError, Role};

use super::interactions::{CommandData, CommandOption, Interaction, Permission, focused, option};

/// The command tree, as Discord wants it registered.
pub fn definition() -> Value {
    json!({
        "name": "wh",
        "description": "Your wormhole maps",
        "options": [
            {
                "type": 1,
                "name": "account",
                "description": "Show which WormholeSystems account this Discord user is linked to"
            },
            alert_group("alert-dm", "Create an alert delivered to you by direct message", false),
            alert_group("alert-channel", "Create an alert posted in this channel", true),
            {
                "type": 2,
                "name": "alerts",
                "description": "The alerts you created",
                "options": [
                    {
                        "type": 1, "name": "list", "description": "List the alerts you created",
                        "options": [{
                            "type": 3, "name": "map", "description": "Only this map",
                            "required": false, "autocomplete": true
                        }]
                    },
                    {
                        "type": 1, "name": "enable", "description": "Turn one back on",
                        "options": [{
                            "type": 3, "name": "alert", "description": "Which alert",
                            "required": true, "autocomplete": true
                        }]
                    },
                    {
                        "type": 1, "name": "disable", "description": "Turn one off",
                        "options": [{
                            "type": 3, "name": "alert", "description": "Which alert",
                            "required": true, "autocomplete": true
                        }]
                    },
                    {
                        "type": 1, "name": "remove", "description": "Delete one",
                        "options": [{
                            "type": 3, "name": "alert", "description": "Which alert",
                            "required": true, "autocomplete": true
                        }]
                    }
                ]
            },
            {
                "type": 1,
                "name": "route",
                "description": "How far a system is from one of your chains",
                "options": [
                    {
                        "type": 3, "name": "map", "description": "Which map",
                        "required": true, "autocomplete": true
                    },
                    {
                        "type": 3, "name": "system", "description": "Where to",
                        "required": true, "autocomplete": true
                    }
                ]
            }
        ]
    })
}

/// One destination's worth of alert subcommands. Discord nests a command at most two deep,
/// so the destination is a group and each kind is a subcommand under it; the channel group
/// carries the mention options, which a direct message has no use for.
fn alert_group(name: &str, description: &str, channel: bool) -> Value {
    let map = json!({
        "type": 3, "name": "map", "description": "Which map",
        "required": true, "autocomplete": true
    });
    let system = json!({
        "type": 3, "name": "system", "description": "Which system to watch",
        "required": true, "autocomplete": true
    });
    let jumps = json!({
        "type": 4, "name": "jumps", "description": "Within this many gate jumps",
        "required": true, "min_value": 1, "max_value": 30
    });
    let ship = json!({
        "type": 3, "name": "ship", "description": "Which hull's range to measure",
        "required": true,
        "choices": JumpShip::ALL.iter()
            .map(|ship| json!({ "name": ship.label(), "value": ship.as_str() }))
            .collect::<Vec<_>>()
    });
    let jdc = json!({
        "type": 4, "name": "jdc", "description": "Jump Drive Calibration level",
        "required": true, "min_value": 0, "max_value": 5
    });
    let from = json!({
        "type": 3, "name": "from", "description": "Measure from this system through the chain",
        "required": false, "autocomplete": true
    });
    let mention = json!({
        "type": 3, "name": "mention", "description": "Who to ping",
        "required": true,
        "choices": [
            { "name": "Nobody", "value": AlertMention::None.as_str() },
            { "name": "Me", "value": AlertMention::Creator.as_str() },
            { "name": "A role", "value": AlertMention::Role.as_str() },
            { "name": "Everyone", "value": AlertMention::Everyone.as_str() }
        ]
    });
    let role = json!({
        "type": 8, "name": "role", "description": "The role to ping", "required": false
    });

    // Discord lists required options before optional ones, and rejects the other order.
    let options = |required: Vec<Value>, optional: Vec<Value>| {
        let mut all = required;
        if channel {
            all.push(mention.clone());
        }
        all.extend(optional);
        if channel {
            all.push(role.clone());
        }
        all
    };
    json!({
        "type": 2,
        "name": name,
        "description": description,
        "options": [
            {
                "type": 1, "name": "proximity",
                "description": "The chain comes within gate jumps of a system",
                "options": options(vec![map.clone(), system.clone(), jumps.clone()], vec![from])
            },
            {
                "type": 1, "name": "jump-range",
                "description": "A k-space exit lands within capital jump range of a system",
                "options": options(vec![map.clone(), system, ship, jdc], vec![])
            },
            {
                "type": 1, "name": "killmail",
                "description": "Something dies within gate jumps of the chain",
                "options": options(vec![map, jumps], vec![])
            }
        ]
    })
}

/// Upload the command tree to Discord, replacing whatever is registered.
///
/// Registered globally rather than per guild, so nothing has to track which servers WormholeSystems
/// is in. The cost is that Discord takes a few minutes to roll a change out.
pub async fn register(application_id: &str, bot_token: &str) -> Result<(), String> {
    let response = crate::user_agent::client()
        .put(format!(
            "{}/applications/{application_id}/commands",
            super::API
        ))
        .header("authorization", format!("Bot {bot_token}"))
        .json(&json!([definition()]))
        .send()
        .await
        .map_err(|err| err.to_string())?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    let body = response.text().await.unwrap_or_default();
    Err(format!("discord returned {status}: {body}"))
}

/// Run a command and produce the reply text.
pub async fn run(state: &AppState, interaction: &Interaction) -> String {
    let Some(sender) = interaction.sender() else {
        return "I could not tell who you are.".into();
    };
    let Some(data) = interaction.data.as_ref() else {
        return "That command arrived empty.".into();
    };
    let user_id = super::user_for(&state.db, &sender.id).await;
    let Some(user_id) = user_id else {
        return unlinked();
    };

    match parse(&data.options) {
        Action::Account => account(state, user_id).await,
        Action::AlertCreate(request) => {
            create_alert(state, user_id, interaction, data, request).await
        }
        Action::AlertsList { map } => alerts(state, user_id, map).await,
        Action::AlertsSetActive { alert, active } => {
            set_active(state, user_id, alert, active).await
        }
        Action::AlertsRemove { alert } => remove(state, user_id, alert).await,
        Action::Route {
            map: Some(map),
            system: Some(system),
        } => route(state, user_id, map, system).await,
        Action::Route { .. } => "Pick a map and a system from the suggestions.".into(),
        Action::Nothing => "Pick one of the subcommands.".into(),
        Action::Unknown(name) => format!("I do not know `{name}`."),
    }
}

/// What an invocation asks for, once the subcommand tree has been walked.
///
/// Discord nests a group's arguments two levels down, and reading them off the top level
/// finds nothing without saying so, which is why walking the tree is its own tested step.
#[derive(Debug, PartialEq)]
enum Action<'a> {
    Account,
    AlertCreate(AlertRequest),
    AlertsList {
        map: Option<&'a str>,
    },
    AlertsSetActive {
        alert: Option<i64>,
        active: bool,
    },
    AlertsRemove {
        alert: Option<i64>,
    },
    Route {
        map: Option<i64>,
        system: Option<i64>,
    },
    /// A name we do not serve: Discord's command list can lag a deploy by minutes.
    Unknown(&'a str),
    Nothing,
}

/// An alert as the command spelled it out, before anything is looked up.
#[derive(Debug, PartialEq)]
struct AlertRequest {
    delivery: AlertDelivery,
    kind: AlertKind,
    map: Option<i64>,
    system: Option<i64>,
    from: Option<i64>,
    jumps: Option<i64>,
    ship: Option<JumpShip>,
    jdc: Option<i64>,
    /// Absent for a direct message, which pings nobody.
    mention: Option<AlertMention>,
    role: Option<String>,
}

fn parse_alert(group: &CommandOption, delivery: AlertDelivery) -> Action<'_> {
    let Some(variant) = group.options.first() else {
        return Action::Nothing;
    };
    let kind = match variant.name.as_str() {
        "proximity" => AlertKind::Proximity,
        "jump-range" => AlertKind::JumpRange,
        "killmail" => AlertKind::Killmail,
        other => return Action::Unknown(other),
    };
    let integer = |name: &str| option(&variant.options, name).and_then(|o| o.integer());
    let string = |name: &str| option(&variant.options, name).and_then(|o| o.string());
    Action::AlertCreate(AlertRequest {
        delivery,
        kind,
        map: integer("map"),
        system: integer("system"),
        from: integer("from"),
        jumps: integer("jumps"),
        ship: string("ship").and_then(JumpShip::from_db),
        jdc: integer("jdc"),
        mention: string("mention").and_then(AlertMention::from_db),
        role: string("role").map(str::to_string),
    })
}

fn parse(options: &[CommandOption]) -> Action<'_> {
    let Some(sub) = options.first() else {
        return Action::Nothing;
    };
    match sub.name.as_str() {
        "account" => Action::Account,
        "alert-dm" => parse_alert(sub, AlertDelivery::DiscordDm),
        "alert-channel" => parse_alert(sub, AlertDelivery::DiscordChannel),
        "alerts" => {
            let Some(action) = sub.options.first() else {
                return Action::Nothing;
            };
            let alert = option(&action.options, "alert").and_then(|o| o.integer());
            match action.name.as_str() {
                "list" => Action::AlertsList {
                    map: option(&action.options, "map").and_then(|o| o.string()),
                },
                "enable" => Action::AlertsSetActive {
                    alert,
                    active: true,
                },
                "disable" => Action::AlertsSetActive {
                    alert,
                    active: false,
                },
                "remove" => Action::AlertsRemove { alert },
                other => Action::Unknown(other),
            }
        }
        "route" => Action::Route {
            map: option(&sub.options, "map").and_then(|o| o.integer()),
            system: option(&sub.options, "system").and_then(|o| o.integer()),
        },
        other => Action::Unknown(other),
    }
}

fn unlinked() -> String {
    "This Discord account is not linked to a WormholeSystems account yet. Open WormholeSystems, go to \
     Settings → Discord, and press Connect."
        .into()
}

async fn account(state: &AppState, user_id: i64) -> String {
    let characters = sqlx::query_scalar!(
        "select name from characters where user_id = $1 order by id",
        user_id,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let maps = sqlx::query_scalar!(
        "select count(distinct m.id) from maps m
         join map_access_live ma on ma.map_id = m.id
         where ma.subject_id in (
             select id from characters where user_id = $1
             union all select corporation_id from characters where user_id = $1
             union all select alliance_id from characters where user_id = $1 and alliance_id is not null
         )",
        user_id,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    if characters.is_empty() {
        return "Linked, but that WormholeSystems account has no characters yet.".into();
    }
    format!(
        "Linked to **{}**{}. {maps} {} you can see.",
        characters[0],
        if characters.len() > 1 {
            format!(" and {} other characters", characters.len() - 1)
        } else {
            String::new()
        },
        if maps == 1 { "map" } else { "maps" }
    )
}

/// The name an alert made from Discord goes by on the settings page, from what it
/// watches, since the command has nowhere to ask for one.
fn alert_name(body: &SaveAlert, target: Option<&str>, origin: Option<&str>) -> String {
    let target = target.unwrap_or("a system");
    match body.kind {
        AlertKind::Proximity => match origin {
            Some(origin) => format!("{target} within {} jumps of {origin}", body.max_jumps),
            None => format!("{target} within {} jumps", body.max_jumps),
        },
        AlertKind::JumpRange => format!(
            "{} range of {target}",
            body.ship_type.map(JumpShip::label).unwrap_or("Capital")
        ),
        AlertKind::Killmail => format!("Kills within {} jumps of the chain", body.max_jumps),
    }
}

async fn system_name(state: &AppState, id: i64) -> Option<String> {
    sqlx::query_scalar!("select name from solar_systems where id = $1", id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
}

/// A role picked in Discord becomes one the map has registered, under the name Discord
/// sent along, so the alert reads on the settings page like one made there.
async fn register_role(
    state: &AppState,
    map_id: i64,
    discord_role_id: &str,
    name: Option<&str>,
) -> Result<i64, crate::maps::MapError> {
    crate::maps::alerts::register_role(
        &state.db,
        map_id,
        name.unwrap_or(discord_role_id),
        discord_role_id,
    )
    .await
}

/// What stands between a channel alert and the channel: the sender's own permissions
/// there. A direct message needs none of this.
fn denied_in_channel(interaction: &Interaction, mention: AlertMention) -> Option<&'static str> {
    let Some(member) = interaction.member.as_ref() else {
        return Some("Channel alerts are created from inside a server channel.");
    };
    if !member.can(Permission::ManageChannels) {
        return Some("You need the Manage Channels permission to create an alert in this channel.");
    }
    if mention == AlertMention::Role && !member.can(Permission::ManageRoles) {
        return Some("You need the Manage Roles permission to mention a role.");
    }
    if mention == AlertMention::Everyone && !member.can(Permission::MentionEveryone) {
        return Some("You need the Mention Everyone permission to mention everyone.");
    }
    None
}

async fn create_alert(
    state: &AppState,
    user_id: i64,
    interaction: &Interaction,
    data: &CommandData,
    request: AlertRequest,
) -> String {
    let Some(map_id) = request.map else {
        return "Pick a map from the suggestions.".into();
    };
    let is_channel = request.delivery == AlertDelivery::DiscordChannel;
    let mention = if is_channel {
        request.mention.unwrap_or(AlertMention::None)
    } else {
        AlertMention::None
    };

    let (guild_id, channel_id) = if is_channel {
        let (Some(guild), Some(channel)) = (&interaction.guild_id, &interaction.channel_id) else {
            return "Channel alerts are created from inside a server channel.".into();
        };
        if let Some(denied) = denied_in_channel(interaction, mention) {
            return denied.into();
        }
        (Some(guild.clone()), Some(channel.clone()))
    } else {
        (None, None)
    };

    // A direct message reaches only its creator, so seeing the map is enough; a channel
    // post reaches a server, which is the manager's call, as it is on the settings page.
    if is_channel {
        match crate::maps::access::require_role(&state.db, map_id, user_id, Role::Manager).await {
            Ok(_) => {}
            Err(MapError::Forbidden) => {
                return "Only map managers can create channel alerts.".into();
            }
            Err(_) => return "You do not have access to that map.".into(),
        }
    } else if !crate::maps::access::can_see(&state.db, map_id, user_id).await {
        return "You do not have access to that map.".into();
    }
    let Some(map_name) = sqlx::query_scalar!("select name from maps where id = $1", map_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
    else {
        return "You do not have access to that map.".into();
    };

    let target = match (request.kind, request.system) {
        (AlertKind::Killmail, _) => None,
        (_, None) => return "Pick a system from the suggestions.".into(),
        (_, Some(id)) => match system_name(state, id).await {
            Some(name) => Some((id, name)),
            None => return "That system is unavailable.".into(),
        },
    };
    let origin = match request.from {
        None => None,
        Some(id) => match system_name(state, id).await {
            Some(name) => Some((id, name)),
            None => return "That starting point is unavailable.".into(),
        },
    };

    let role_id = match (request.role.as_deref(), mention) {
        (Some(role), AlertMention::Role) => {
            match register_role(state, map_id, role, data.role_name(role)).await {
                Ok(id) => Some(id),
                Err(err) => {
                    eprintln!("discord: registering role {role} on map {map_id}: {err}");
                    return "I could not register that role just now.".into();
                }
            }
        }
        (Some(_), _) => return "Pick a role only when the mention is A role.".into(),
        (None, _) => None,
    };

    let jumps = match (request.kind, request.jumps) {
        (AlertKind::JumpRange, _) => 0,
        (_, Some(jumps)) => jumps as i32,
        (_, None) => return "Say how many jumps.".into(),
    };
    let mut body = SaveAlert {
        name: String::new(),
        kind: request.kind,
        delivery: request.delivery,
        map_webhook_id: None,
        discord_guild_id: guild_id,
        discord_channel_id: channel_id,
        map_webhook_role_id: role_id,
        mention,
        target_solar_system_id: target.as_ref().map(|(id, _)| *id),
        origin_solar_system_id: origin.as_ref().map(|(id, _)| *id),
        max_jumps: jumps,
        ship_type: request.ship,
        jdc_level: request.jdc.map(|level| level as i32),
        filters: Vec::new(),
        filter_match: filters::Match::Any,
    };
    let target_name = target.as_ref().map(|(_, name)| name.as_str());
    let origin_name = origin.as_ref().map(|(_, name)| name.as_str());
    body.name = alert_name(&body, target_name, origin_name);

    let saved = match crate::maps::alerts::check(&state.db, map_id, &body).await {
        Ok(()) => crate::maps::alerts::create(&state.db, map_id, user_id, &body).await,
        Err(err) => Err(err),
    };
    match saved {
        Ok(_) => {}
        Err(MapError::Validation(message)) => {
            return format!("That alert is not right: {message}.");
        }
        Err(_) => return "I could not save that alert just now.".into(),
    }

    let target_name = target_name.unwrap_or("a system");
    match (body.kind, origin_name) {
        (AlertKind::Proximity, Some(origin)) => format!(
            "Alert created for **{target_name}** within {jumps} jumps of **{origin}** through the **{map_name}** chain."
        ),
        (AlertKind::Proximity, None) => {
            format!("Alert created for **{target_name}** within {jumps} jumps of **{map_name}**.")
        }
        (AlertKind::JumpRange, _) => format!(
            "Alert created for exits within {:.1} ly of **{target_name}** on **{map_name}**.",
            body.ship_type
                .map(|ship| ship.max_range_ly(body.jdc_level.unwrap_or(0)))
                .unwrap_or(0.0)
        ),
        (AlertKind::Killmail, _) => {
            format!("Alert created for kills within {jumps} jumps of the **{map_name}** chain.")
        }
    }
}

async fn alerts(state: &AppState, user_id: i64, map_filter: Option<&str>) -> String {
    let map_id: Option<i64> = map_filter.and_then(|value| value.parse().ok());
    let rows = sqlx::query!(
        "select a.name, a.kind, a.is_active, a.disabled_reason, m.name as map_name
         from map_alerts a
         join maps m on m.id = a.map_id
         where a.created_by_user_id = $1 and ($2::bigint is null or a.map_id = $2)
         order by m.name, a.name",
        user_id,
        map_id,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return "You have not created any alerts.".into();
    }
    let mut out = String::from("**Your alerts**\n");
    for row in rows {
        let state_text = if row.is_active {
            "on".to_string()
        } else {
            format!(
                "off ({})",
                row.disabled_reason.as_deref().unwrap_or("manual")
            )
        };
        out.push_str(&format!(
            "• `{}` {} on **{}** ({state_text})\n",
            row.kind, row.name, row.map_name
        ));
    }
    out
}

/// The alert, if this user created it. Ownership is the permission for these commands.
async fn owned(state: &AppState, user_id: i64, alert_id: i64) -> Option<(i64, String, i64)> {
    sqlx::query!(
        "select id, name, map_id from map_alerts where id = $1 and created_by_user_id = $2",
        alert_id,
        user_id,
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .map(|row| (row.id, row.name, row.map_id))
}

async fn set_active(state: &AppState, user_id: i64, alert: Option<i64>, active: bool) -> String {
    let Some(alert_id) = alert else {
        return "Pick an alert from the suggestions.".into();
    };
    let Some((id, name, map_id)) = owned(state, user_id, alert_id).await else {
        return "That is not one of your alerts.".into();
    };
    match crate::maps::alerts::set_active(
        &state.db,
        map_id,
        id,
        user_id,
        active,
        Some("via Discord"),
    )
    .await
    {
        Ok(_) => format!("**{name}** is now {}.", if active { "on" } else { "off" }),
        Err(err) => store_failed(&name, err),
    }
}

async fn remove(state: &AppState, user_id: i64, alert: Option<i64>) -> String {
    let Some(alert_id) = alert else {
        return "Pick an alert from the suggestions.".into();
    };
    let Some((id, name, map_id)) = owned(state, user_id, alert_id).await else {
        return "That is not one of your alerts.".into();
    };
    match crate::maps::alerts::delete(&state.db, map_id, id, user_id).await {
        Ok(()) => format!("Deleted **{name}**."),
        Err(err) => store_failed(&name, err),
    }
}

/// The reply when a change to an alert did not stick. The cause goes to the log rather
/// than the channel: a database error means nothing to whoever typed the command.
fn store_failed(name: &str, err: crate::maps::MapError) -> String {
    match err {
        crate::maps::MapError::NotFound => format!("**{name}** is gone already."),
        err => {
            eprintln!("discord: changing alert {name}: {err}");
            format!("I could not change **{name}** just now. Try again in a moment.")
        }
    }
}

async fn route(state: &AppState, user_id: i64, map_id: i64, system_id: i64) -> String {
    if !crate::maps::access::can_see(&state.db, map_id, user_id).await {
        return "You do not have access to that map.".into();
    }
    let Some(chain) = crate::alerts::killmail::chain_of(&state.db, map_id).await else {
        return "That map has no systems on it yet.".into();
    };
    let Ok(universe) = crate::alerts::proximity::Universe::load(&state.db).await else {
        return "I could not read the star map just now.".into();
    };
    let Some(found) = crate::alerts::proximity::nearest(
        &universe,
        &chain.systems,
        &chain.edges,
        &chain.ignored,
        system_id,
        // Beyond this nobody is flying it anyway, and the search stays bounded.
        30,
    ) else {
        return "That system is more than 30 jumps from the chain.".into();
    };

    let names = sqlx::query!(
        "select id, name from solar_systems where id = any($1)",
        &found.route,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let named = |id: i64| {
        names
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| id.to_string())
    };
    let route: Vec<String> = found.route.iter().map(|id| named(*id)).collect();
    format!(
        "**{}** is {} {} from **{}**.\n{}",
        named(system_id),
        found.jumps,
        if found.jumps == 1 { "jump" } else { "jumps" },
        named(found.from),
        route.join(" → ")
    )
}

/// Suggestions for whichever option is being typed.
pub async fn autocomplete(state: &AppState, interaction: &Interaction) -> Vec<Value> {
    let Some(data) = interaction.data.as_ref() else {
        return Vec::new();
    };
    let Some(field) = focused(&data.options) else {
        return Vec::new();
    };
    let typed = field.string().unwrap_or("").trim().to_lowercase();
    let Some(sender) = interaction.sender() else {
        return Vec::new();
    };
    let Some(user_id) = super::user_for(&state.db, &sender.id).await else {
        return Vec::new();
    };

    match field.name.as_str() {
        "map" => maps_for(state, user_id, &typed).await,
        "system" | "from" => systems_like(state, &typed).await,
        "alert" => alerts_for(state, user_id, &typed).await,
        _ => Vec::new(),
    }
}

/// Discord shows at most 25 choices, so ask for that many and no more.
const CHOICES: i64 = 25;

async fn maps_for(state: &AppState, user_id: i64, typed: &str) -> Vec<Value> {
    let like = format!("%{typed}%");
    let rows = sqlx::query!(
        "select distinct m.id, m.name from maps m
         join map_access_live ma on ma.map_id = m.id
         where m.name ilike $2 and ma.subject_id in (
             select id from characters where user_id = $1
             union all select corporation_id from characters where user_id = $1
             union all select alliance_id from characters where user_id = $1 and alliance_id is not null
         )
         order by m.name
         limit $3",
        user_id,
        like,
        CHOICES,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    rows.into_iter()
        // The value is the id as a string: Discord's autocomplete values are strings, and
        // the command parses them back.
        .map(|row| json!({ "name": row.name, "value": row.id.to_string() }))
        .collect()
}

/// Only the sender's own alerts: they are the only ones these commands can act on.
async fn alerts_for(state: &AppState, user_id: i64, typed: &str) -> Vec<Value> {
    let like = format!("%{typed}%");
    let rows = sqlx::query!(
        "select a.id, a.name, a.is_active, m.name as map_name
         from map_alerts a join maps m on m.id = a.map_id
         where a.created_by_user_id = $1 and a.name ilike $2
         order by m.name, a.name
         limit $3",
        user_id,
        like,
        CHOICES,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|row| {
            json!({
                "name": format!(
                    "{}, {}{}",
                    row.name,
                    row.map_name,
                    if row.is_active { "" } else { " (off)" }
                ),
                "value": row.id.to_string()
            })
        })
        .collect()
}

async fn systems_like(state: &AppState, typed: &str) -> Vec<Value> {
    if typed.len() < 2 {
        return Vec::new();
    }
    let contains = format!("%{typed}%");
    let prefix = format!("{typed}%");
    let rows = sqlx::query!(
        "select s.id, s.name, r.name as region
         from solar_systems s join regions r on r.id = s.region_id
         where s.name ilike $1
         order by (s.name ilike $2) desc, length(s.name), s.name
         limit $3",
        contains,
        prefix,
        CHOICES,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|row| {
            json!({
                "name": format!("{}, {}", row.name, row.region),
                "value": row.id.to_string()
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The registered shape is what Discord validates on upload, so it is worth pinning.
    #[test]
    fn the_command_registers_as_one_name_with_subcommands() {
        let definition = definition();
        assert_eq!(definition["name"], "wh");
        let subs = definition["options"].as_array().unwrap();
        assert_eq!(subs.len(), 5);
        for sub in subs {
            // Type 1 is a subcommand, type 2 a group of them.
            assert!(sub["type"] == 1 || sub["type"] == 2);
            assert!(sub["description"].as_str().unwrap().len() > 5);
        }
        // The alerts group is what makes an alert manageable from Discord.
        let alerts = subs.iter().find(|s| s["name"] == "alerts").unwrap();
        let actions: Vec<&str> = alerts["options"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["name"].as_str().unwrap())
            .collect();
        assert_eq!(actions, vec!["list", "enable", "disable", "remove"]);
    }

    fn options(json: &str) -> Vec<CommandOption> {
        serde_json::from_str::<super::super::interactions::CommandData>(json)
            .unwrap()
            .options
    }

    /// The arguments of a subcommand group sit two levels down. Read off the top level they
    /// come back empty, and the action quietly falls through to the list.
    #[test]
    fn an_alert_action_is_read_from_inside_its_group() {
        let enable = options(
            r#"{"name":"wh","options":[{"name":"alerts","type":2,"options":[
                 {"name":"enable","type":1,"options":[{"name":"alert","value":"42"}]}]}]}"#,
        );
        assert_eq!(
            parse(&enable),
            Action::AlertsSetActive {
                alert: Some(42),
                active: true
            }
        );

        let disable = options(
            r#"{"name":"wh","options":[{"name":"alerts","type":2,"options":[
                 {"name":"disable","type":1,"options":[{"name":"alert","value":"42"}]}]}]}"#,
        );
        assert_eq!(
            parse(&disable),
            Action::AlertsSetActive {
                alert: Some(42),
                active: false
            }
        );

        let remove = options(
            r#"{"name":"wh","options":[{"name":"alerts","type":2,"options":[
                 {"name":"remove","type":1,"options":[{"name":"alert","value":"42"}]}]}]}"#,
        );
        assert_eq!(parse(&remove), Action::AlertsRemove { alert: Some(42) });
    }

    /// The same nesting: without it `list` filters by nothing whatever map was picked.
    #[test]
    fn listing_alerts_keeps_the_map_it_was_filtered_by() {
        let filtered = options(
            r#"{"name":"wh","options":[{"name":"alerts","type":2,"options":[
                 {"name":"list","type":1,"options":[{"name":"map","value":"7"}]}]}]}"#,
        );
        assert_eq!(parse(&filtered), Action::AlertsList { map: Some("7") });

        let all = options(
            r#"{"name":"wh","options":[{"name":"alerts","type":2,"options":[
                 {"name":"list","type":1}]}]}"#,
        );
        assert_eq!(parse(&all), Action::AlertsList { map: None });
    }

    /// A plain subcommand carries its own arguments, one level down rather than two.
    #[test]
    fn a_plain_subcommand_reads_its_own_arguments() {
        let route = options(
            r#"{"name":"wh","options":[{"name":"route","type":1,"options":[
                 {"name":"map","value":"7"},{"name":"system","value":"30000142"}]}]}"#,
        );
        assert_eq!(
            parse(&route),
            Action::Route {
                map: Some(7),
                system: Some(30000142)
            }
        );
        assert_eq!(
            parse(&options(r#"{"name":"wh","options":[]}"#)),
            Action::Nothing
        );
        assert_eq!(
            parse(&options(
                r#"{"name":"wh","options":[{"name":"wat","type":1}]}"#
            )),
            Action::Unknown("wat")
        );
    }

    fn names(options: &Value) -> Vec<&str> {
        options
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["name"].as_str().unwrap())
            .collect()
    }

    /// Discord rejects a required option after an optional one, and validates the whole
    /// tree on upload, so the exact option order is worth pinning per variant.
    #[test]
    fn each_destination_offers_every_kind_with_only_its_own_options() {
        let definition = definition();
        let subs = definition["options"].as_array().unwrap();
        for (group, channel) in [("alert-dm", false), ("alert-channel", true)] {
            let group = subs.iter().find(|s| s["name"] == group).unwrap();
            assert_eq!(group["type"], 2);
            assert_eq!(
                names(&group["options"]),
                vec!["proximity", "jump-range", "killmail"]
            );
            let variant = |name: &str| {
                group["options"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["name"] == name)
                    .unwrap()
                    .clone()
            };
            let expected = |own: &[&'static str], optional: &[&'static str]| {
                let mut all: Vec<&'static str> = own.to_vec();
                if channel {
                    all.push("mention");
                }
                all.extend(optional);
                if channel {
                    all.push("role");
                }
                all
            };
            assert_eq!(
                names(&variant("proximity")["options"]),
                expected(&["map", "system", "jumps"], &["from"])
            );
            assert_eq!(
                names(&variant("jump-range")["options"]),
                expected(&["map", "system", "ship", "jdc"], &[])
            );
            assert_eq!(
                names(&variant("killmail")["options"]),
                expected(&["map", "jumps"], &[])
            );
            for variant in group["options"].as_array().unwrap() {
                let mut seen_optional = false;
                for option in variant["options"].as_array().unwrap() {
                    let required = option["required"] == true;
                    assert!(
                        !(required && seen_optional),
                        "{}: required after optional",
                        variant["name"]
                    );
                    seen_optional |= !required;
                }
            }
        }
        let ship =
            &subs.iter().find(|s| s["name"] == "alert-dm").unwrap()["options"][1]["options"][2];
        assert_eq!(ship["name"], "ship");
        assert_eq!(
            ship["choices"].as_array().unwrap().len(),
            JumpShip::ALL.len()
        );
    }

    /// The destination is the group and the kind the subcommand, so both are read off
    /// the tree and every option comes from the leaf.
    #[test]
    fn an_alert_request_is_read_from_its_destination_and_kind() {
        let channel = options(
            r#"{"name":"wh","options":[{"name":"alert-channel","type":2,"options":[
                 {"name":"proximity","type":1,"options":[
                     {"name":"map","value":"7"},{"name":"system","value":"30000142"},
                     {"name":"jumps","value":5},{"name":"mention","value":"role"},
                     {"name":"from","value":"30000144"},{"name":"role","value":"77"}]}]}]}"#,
        );
        assert_eq!(
            parse(&channel),
            Action::AlertCreate(AlertRequest {
                delivery: AlertDelivery::DiscordChannel,
                kind: AlertKind::Proximity,
                map: Some(7),
                system: Some(30000142),
                from: Some(30000144),
                jumps: Some(5),
                ship: None,
                jdc: None,
                mention: Some(AlertMention::Role),
                role: Some("77".into()),
            })
        );

        let dm = options(
            r#"{"name":"wh","options":[{"name":"alert-dm","type":2,"options":[
                 {"name":"jump-range","type":1,"options":[
                     {"name":"map","value":"7"},{"name":"system","value":"30000142"},
                     {"name":"ship","value":"carrier"},{"name":"jdc","value":4}]}]}]}"#,
        );
        assert_eq!(
            parse(&dm),
            Action::AlertCreate(AlertRequest {
                delivery: AlertDelivery::DiscordDm,
                kind: AlertKind::JumpRange,
                map: Some(7),
                system: Some(30000142),
                from: None,
                jumps: None,
                ship: Some(JumpShip::Carrier),
                jdc: Some(4),
                mention: None,
                role: None,
            })
        );

        let stale = options(
            r#"{"name":"wh","options":[{"name":"alert-dm","type":2,"options":[
                 {"name":"skyhook","type":1}]}]}"#,
        );
        assert_eq!(parse(&stale), Action::Unknown("skyhook"));
    }

    fn body(kind: AlertKind) -> SaveAlert {
        SaveAlert {
            name: String::new(),
            kind,
            delivery: AlertDelivery::DiscordDm,
            map_webhook_id: None,
            discord_guild_id: None,
            discord_channel_id: None,
            map_webhook_role_id: None,
            mention: AlertMention::None,
            target_solar_system_id: None,
            origin_solar_system_id: None,
            max_jumps: 5,
            ship_type: Some(JumpShip::Dreadnought),
            jdc_level: Some(5),
            filters: Vec::new(),
            filter_match: filters::Match::Any,
        }
    }

    /// The name is what the settings page and `/wh alerts list` show, so it has to say
    /// what the alert watches without the person having typed anything.
    #[test]
    fn an_alert_made_from_discord_is_named_after_what_it_watches() {
        assert_eq!(
            alert_name(&body(AlertKind::Proximity), Some("Jita"), None),
            "Jita within 5 jumps"
        );
        assert_eq!(
            alert_name(&body(AlertKind::Proximity), Some("Jita"), Some("J123456")),
            "Jita within 5 jumps of J123456"
        );
        assert_eq!(
            alert_name(&body(AlertKind::JumpRange), Some("Jita"), None),
            "Dreadnought range of Jita"
        );
        assert_eq!(
            alert_name(&body(AlertKind::Killmail), None, None),
            "Kills within 5 jumps of the chain"
        );
    }

    fn interaction(json: &str) -> Interaction {
        serde_json::from_str(json).unwrap()
    }

    /// Discord itself has already rejected the command for someone who cannot see the
    /// channel; what it does not check is whether they may speak for it.
    #[test]
    fn a_channel_alert_needs_the_senders_own_permissions_there() {
        let with = |permissions: u64| {
            interaction(&format!(
                r#"{{"type":2,"guild_id":"1","channel_id":"2","member":{{"permissions":"{permissions}"}}}}"#
            ))
        };
        let manage_channels = Permission::ManageChannels as u64;
        assert!(denied_in_channel(&with(0), AlertMention::None).is_some());
        assert!(denied_in_channel(&with(manage_channels), AlertMention::None).is_none());
        assert!(denied_in_channel(&with(manage_channels), AlertMention::Creator).is_none());
        assert!(denied_in_channel(&with(manage_channels), AlertMention::Role).is_some());
        assert!(denied_in_channel(&with(manage_channels), AlertMention::Everyone).is_some());
        assert!(
            denied_in_channel(
                &with(manage_channels | Permission::ManageRoles as u64),
                AlertMention::Role
            )
            .is_none()
        );
        assert!(
            denied_in_channel(
                &with(Permission::Administrator as u64),
                AlertMention::Everyone
            )
            .is_none()
        );
        let in_dm = interaction(r#"{"type":2,"user":{"id":"42","username":"pilot"}}"#);
        assert!(denied_in_channel(&in_dm, AlertMention::None).is_some());
    }

    #[test]
    fn route_and_alerts_offer_autocomplete_where_a_name_is_wanted() {
        let definition = definition();
        let subs = definition["options"].as_array().unwrap();
        let route = subs.iter().find(|s| s["name"] == "route").unwrap();
        for option in route["options"].as_array().unwrap() {
            assert_eq!(option["autocomplete"], true);
            assert_eq!(option["required"], true);
        }
    }
}
