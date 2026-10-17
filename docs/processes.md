# Background Processes

Scheduled / background work the server runs outside of request handling. One section
per job: its purpose, triggers, and the data it touches.

## Affiliation refresh

Keeps each [`characters`](./database/authentication.md#characters) row's
`corporation_id` / `alliance_id` current. This matters because those fields **drive
[access](./database/access.md)**: a map can grant access to a whole corporation or
alliance, so when a character's affiliation changes, what maps they can reach must
change too. Both triggers use the bulk
[character affiliation](./esi/characters-affiliation.md) endpoint.

### On authentication

When a character signs in, or is added to an account, the login fetches that
character's affiliation and stores its corp/alliance **before** opening the session. So
someone who just left a corporation can't get in on stale data at login. The account's
other characters are left to the periodic job.

### Periodic (the background job)

Users who don't log in would otherwise keep stale affiliations indefinitely, so a
background loop (`affiliation::start`) refetches the affiliations of **all** characters
assigned to users, hourly, skipping a tick while Tranquility is down:

- Batch every character id through `POST /characters/affiliation` (≤ 1000 ids per
  call), update `corporation_id` / `alliance_id`, and bump `updated_at`.
- Corporations and alliances not seen before are fetched first (`entities::ensure`),
  since the columns are foreign keys to them. One that cannot be fetched is stored as
  `null` until the next run: less access for an hour, never more.
- A character that has left its corporation/alliance stops matching that corp/alliance
  `map_access` grant on the next run → **access is lost within one refresh interval.**
- A batch ESI refuses is skipped and retried next run; the rest of the run goes on.

**Why every account character, not just the active ones.** A session is not the only
way access is used: personal access tokens, the Discord bot and alerts all act for a
user with no session open. The bulk endpoint makes the whole set cheap, one public call
per thousand characters. Characters only named from killmails (no `user_id`) are left
out: no access hangs on them.

**Why hourly.** ESI caches an affiliation for an hour, so asking more often returns the
same answer. Worst case, someone who leaves a corp keeps its maps for about two hours:
ESI's cache, then our next run.

**Invariants & expected behaviour**

- After a run, every account character's stored corp/alliance equals what ESI reports
  (or `null` for an organisation that could not be named).
- A character no longer in a granted corporation/alliance loses that
  affiliation-derived access at the next run — bounded staleness equals the interval.
- **Direct character-id grants are unaffected** by affiliation changes; they persist
  until explicitly removed.
- Map WebSockets check access when they connect, again whenever the map's grants or
  sharing change, and once a minute besides, so an open map that lost its access is
  closed (code `4403`) within a minute of the refresh that took it away.

## Character status polling

Keeps [`character_status`](./database/tracking.md#character_status) live for characters
whose user is actively using the app — without burning ESI calls on idle users or
offline characters. A **two-tier** poll, gated by app activity.

**Eligibility — active users only.** We poll only the characters of a user who has
interacted with the app **within the last 5 minutes**, measured by
[`users.last_active_at`](./database/authentication.md#users). When a user goes idle
past that window, *all* polling for their characters stops.

**Tier 1 — online state, every 60 s.** For each eligible character, call
[`GET …/online`](./esi/character-online.md) (`esi-location.read_online.v1`) and update
`online` / `last_online_at`.

**Tier 2 — location + ship, every 5 s.** *Only for characters currently online*, call
[`GET …/location`](./esi/character-location.md) and
[`GET …/ship`](./esi/character-ship.md), updating `solar_system_id` +
station/structure (→ derived `is_docked`) and the `ship_*` fields.

**Transitions**

- A character seen **offline** in Tier 1 drops out of Tier 2 until it is online again.
- A character that comes **online** in Tier 1 enters Tier 2 on the next 5 s tick.
- A user that goes **idle** (>5 min) removes all their characters from both tiers.
- Every Tier 1 tick first marks **offline** each character still flagged online that is
  no longer polled (user idle, or token refused), and tells its maps. A pilot is on a map
  only while their user has the app open, as in legacy.

**Invariants & expected behaviour**

- A character is in Tier 2 **iff** it is online *and* its user is active (<5 min).
- An offline character is never polled for location/ship.
- An idle user's characters are not polled at all (neither tier).
- Each poll needs the matching scope (online / location / ship); a missing scope skips
  that field, which stays `null` (see [scopes](./database/authentication.md#esi_scopes)).
- Cadences track ESI's cache windows — honour each response's cache-expiry / `ETag`
  rather than re-requesting inside a cached window (see [ESI](./esi/README.md)).
- A poll loads the character's token once for all its fields, from an in-memory cache
  that goes back to the database only to refresh it (or, once a minute at most, to look
  for a scope it lacks). Refreshes of one character's token never run side by side.
- A refresh EVE answers with `invalid_grant` marks the token rejected, logged once at
  warn. The character drops out of both tiers until it logs in again, and the characters
  page asks it to.
- Only changes are written. The poller holds at most half the database pool, so a busy
  tick cannot starve requests.
- Neither tier polls while Tranquility is down, nor from 11:00 to 11:05 UTC, when ESI
  starts failing before `/status` says so.
- A tick with failed polls logs one warning. Only a tier that has failed on every tick
  for 15 minutes logs an error, once.

## Sovereignty refresh

Keeps [`system_sovereignty`](./database/universe.md#system_sovereignty) current — who
holds each K-space system — for display on the map.

- Periodically pull [`GET /sovereignty/systems`](./esi/sovereignty-systems.md) (public,
  whole-of-New-Eden in one call) and upsert one row per claimed system: alliance +
  corporation, or faction; clear systems that became unclaimed.
- Sovereignty changes slowly relative to presence, so this runs on a coarse cadence
  (not the per-second tier of status polling).

**Invariants & expected behaviour**

- After a run, every system's stored holder equals what ESI reports; flipped systems
  reflect their new owner; unclaimed systems carry no holder.

> **Open — interval.** Refresh cadence for sovereignty (e.g. every few minutes vs.
> hourly) — slower than presence, but how slow?

## Connection life-cycle

Wormholes die on a clock, and a chain nobody is flying is not re-scanned. One loop, every
10 minutes, does what a pilot would: marks the holes that must be running out, and drops
the ones that cannot still be there. Both go through the command layer as the system
actor, so each change is an audit entry in the map's history (never an undo step) and
open maps refetch it like any edit. Mirrors legacy's `CheckConnectionAge` and the
connection half of its `DeleteOldSignatures`.

### Ageing

For every wormhole edge not already `critical`, the mark its age has earned:

| Hole | Lifetime | `eol` from | `critical` from |
|---|---|---|---|
| any wormhole | 24 h | 20 h | 23 h |
| C6 to known space | 48 h | 44 h | 47 h |
| drifter (C14 to C18) to known space | 16 h | 12 h | 15 h |

Age is `now - created_at` (an import keeps the connection's original `connected_at`
there). Known space is class 7, 8 or 9. A ghost endpoint has no class, so a hole into one
runs on the 24 h clock.

An edge already at `eol` is judged from its mark instead: `critical` once
`time_status_updated_at` is 3 h old. A pilot's EOL mark often comes from a scan of a hole
older than the map knows, so the mark beats the age.

**Invariants**

- Only escalates. `stable`, `eol` and `critical` marks set by a pilot are never lowered,
  and an unknown (`null`) hole stays unknown until it earns `eol`.
- Stops at `critical`. `expired` (in game "Expired, closure imminent") is only ever set
  by a pilot, and an expired hole is left alone.
- The mark lands on the edge; the `map_conn_sync` trigger carries it to the linked
  signatures and stamps `time_status_updated_at`.
- Stargate edges never age.
- One `connections.aged` entry per mark, with no character.
- The lifetime the class pair gives a hole rides along on the connection as
  `lifetime_hours`, so the client can count down: from a mark, 4 h (`eol`) or 1 h
  (`critical`) after `time_status_updated_at`; before one, at most `created_at` plus the
  lifetime, since when a hole was mapped is at best when it opened. Every wormhole gets
  a countdown, marked or not, so the estimate reads 4 h and 1 h left at the ages the
  loop marks it; past the deadline it reads "any moment now".

### Expiry

No wormhole lives 3 days. Every wormhole edge whose `created_at` is older than that is
removed, along with the placements it strands (the same orphan rule as the manual stale
sweep: unpinned, not home, not rally, and now edgeless). One `connections.expired` entry
per map per run.

The signatures it was linked to are unlinked, not deleted; the
[signature expiry](#see-also--other-periodic-work) collects them on its own schedule,
since a wormhole signature that old is past its 3 day cutoff too.

## Discord alerts

Evaluating an alert and sending its message are separate, so a slow or dead Discord
costs messages and never holds up anything else (`src/alerts/mod.rs`).

- Evaluation decides which alerts fire and claims each occasion in
  `map_alert_deliveries` before anything is sent. Map changes are evaluated by a
  subscriber to every map's events (a placement for proximity and jump range, a wormhole
  for proximity); killmails by a worker the ingest hands them to.
- The active alerts are cached in memory. Any change logged to `map_alert_events` (which
  every write to an alert or its destination is) invalidates the copy at once, and it is
  reloaded at least every minute regardless.
- Messages go to a delivery queue of 256, sent four at a time. When it is full the new
  message is dropped with a warning and its claim handed back: a backlog that deep means
  Discord is down or throttling, and news that old is not worth sending late.
- The event subscriber can still fall behind the 128-event channel. It logs that and
  replays every placement and wormhole made since the last event it took (with 30 s of
  slack, at most 10 minutes) on maps with a proximity or jump-range alert. The claims
  make the replay harmless for what was already handled.

## Retention sweep

Once a day (`retention::sweep`), rows that only ever accumulate are dropped:

| Table | Removed | Why that long |
|---|---|---|
| `map_alert_deliveries` | `killmail:` keys older than 30 days; `placement:` keys older than 30 days whose placement no longer exists | a delivery is the dedup record, so it stays while its occasion can recur; a killmail is only offered once |
| `map_alert_events` | older than 90 days | the alert settings page shows the newest 50 to 200 |
| `sessions` | past `expires_at` | already refused at the door |

Delivery keys of any other shape are kept. A placement removed and then restored by undo
keeps its id, so a delivery swept after the removal can let that alert fire once more; that
takes a removal undone within the 7 day history of a delivery at least 30 days old.

`oauth_login_flows` is cleaned separately (see below).

## See also — other periodic work

Documented with their own tables; listed here so the background-work picture is in one
place:

- Access-token refresh from the stored refresh token
  ([`esi_tokens`](./database/authentication.md#esi_tokens)).
- Cleanup of expired
  [`oauth_login_flows`](./database/authentication.md#oauth_login_flows), done whenever a
  new handshake starts rather than on a timer.
- Signature expiry: unlinked wormhole signatures older than 3 days and other signatures
  untouched for 7 days are deleted (`maps::signatures::expire_signatures`).
- Pruning of connection-jump observations no connection claimed within 10 minutes
  ([`map_connection_jumps`](./database/mapping.md#map_connection_jumps)).
- Naming the characters, corporations and alliances that killmails and sovereignty only
  give ids for. For killmails, every minute, over the kills the live stream stored since
  the last pass (`killmails.ingest_seq` past `zkb_state.named_seq`, within the card's
  7-day window): organisations one at a time through `entities::ensure` (the card wants
  their tickers), pilots a thousand at a time through `/universe/names`. A batch ESI
  fails on is retried for three passes before the mark moves past it. Every kill in EVE
  is named, not only those in mapped systems: a system mapped later should show its week
  of kills with names at once, and alerts, the access search and threat analysis read the
  same names. Ids ESI no longer knows are noted in
  [`unresolvable_entities`](./database/universe.md#unresolvable_entities) and left alone
  for 30 days rather than asked for on every pass; a bulk call one of them fails is
  halved until the dead id is found.
