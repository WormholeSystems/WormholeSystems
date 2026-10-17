# Mapping

The live map graph: solar systems placed on a map and the wormhole / stargate
connections between them. Part of the [database spec](./README.md) — see it for the
conventions and goals these tables serve.

## `maps`

The top-level artifact. A user creates a map and everything else hangs off it.

| Column        | Type        | Notes                                                       |
|---------------|-------------|-------------------------------------------------------------|
| `id`          | pk          |                                                             |
| `name`        | text        | user-chosen                                                 |
| `description` | text, null  | free-text, user-set                                         |
| `image_url`   | text, null  | reference to an uploaded icon/logo, **not** the image bytes |
| `ghost_unlinked_wormholes` | bool | a wormhole signature with no connection puts a ghost placement on the map |
| `layout` | text | `manual` (dragged into shape) or `tree` (drawn from the connections) |
| `allow_layout_override` | bool | whether a viewer may pick their own placement instead |
| `created_at`  | timestamptz |                                                             |

**Invariants & expected behaviour**

- A map always has at least one **owner**, recorded as a `role = owner` row in
  [`map_access`](./access.md#map_access) (there is no `owner_id` column).
- On creation, the map gets an `owner` access grant for its creator's character.
- The icon/logo is **uploaded to object storage** (S3-style or a static asset dir);
  `image_url` holds only the reference. We do not store image bytes in Postgres.
- **Automatic placement derives positions, it does not store them.** A map on `tree`
  keeps every `position_x` / `position_y` exactly as it was left, so switching back finds
  the chain as people dragged it. The tree is computed on the client from the connections,
  rooted at the pinned systems (see
  [map-canvas.md](../legacy/map-canvas.md#8-layout-modes-the-second-rendering-mode)).
- A viewer's own choice lives in `map_user_settings.layout_override` and only counts while
  the map sets `allow_layout_override`; `null` follows the map.
- Deleting a map cascades to its solar systems, connections, signatures, details,
  and access entries (nothing dangles). Removing the map should also clean up its
  uploaded image from object storage (app-level, outside the DB transaction).

---

## `map_solar_systems`

*Ephemeral placement.* A solar system *as currently placed on a map*. Holds only data that is meaningful
while the system is on the map; removing the system deletes this row.

| Column            | Type        | Notes                                            |
|-------------------|-------------|--------------------------------------------------|
| `id`              | pk          |                                                  |
| `map_id`          | fk maps     |                                                  |
| `solar_system_id` | int, null   | → [`solar_systems`](./universe.md#solar_systems); null = a **ghost** |
| `position_x`      | int/float   | grid position on the map                         |
| `position_y`      | int/float   | grid position on the map                         |
| `alias`           | text, null  | temporary, user-set; lost on removal             |
| `raised_by_signature_id` | fk signatures, null | the scan this ghost is the far side of; cascade |
| `hangs_off_id`    | fk self, null | the placement the scan was made in; cascade    |
| `signatures_pasted_at` | timestamptz, null | when a scan was last pasted into the system; only a paste sets it, undo puts the previous time back |
| `created_at`      | timestamptz |                                                  |

### Ghost placements

A wormhole signature is known before the system behind it is. A **ghost** is that far
side on the map: a placement with **no** `solar_system_id`, hanging off the system the
signature was scanned in, so a chain can be laid out and named before anyone flies it.
Any way of saying a hole is there raises one — a pasted scan, a signature typed in by
hand, or an existing signature recategorised as a wormhole.

Ghosts are ordinary placements deliberately, not a table of their own: connections
already reference `map_solar_systems.id` rather than a solar system, so a ghost gets
edges, a position, an alias, and the connection's life-cycle bookkeeping with no second
kind of node anywhere. What sets one apart is the two columns it fills in and a real
system leaves null — `raised_by_signature_id` and `hangs_off_id`, the scan it was drawn
for and the placement that scan was made in. A check constraint makes those the two
shapes a row is allowed to take, and both foreign keys cascade, so the two rules that
govern a ghost's life are the database's rather than any one write's: the scan goes, the
node goes; the system goes, what hung off it goes. What a ghost cannot have is anything
keyed by *system*:
[`signatures`](#signatures) and
[`map_solar_system_details`](#map_solar_system_details) both hang off
`(map_id, solar_system_id)`, so a ghost holds no scan and no intel until it is resolved.

**Invariants & expected behaviour**

- Unique `(map_id, solar_system_id)` — a system appears at most once per map. Nulls are
  distinct in Postgres, so this caps real systems without capping ghosts.
- A system always has an `(x, y)` position while placed.
- A map has at most one **home** (a partial unique index) and at most two **rally
  points**. A rally point is a `rallied_at` timestamp (`is_rally` is generated from it),
  so setting a third can clear the oldest. The cap is the app's: an index can only say
  one.
- A ghost is never **home**, **rally** or **pinned** (a check constraint). The first two
  mean a place you can go; pinning means a place you have decided matters, which holds the
  node still, roots the tree layout and is passed over by every sweep — that last one
  would let a ghost outlive the connection it is the far side of.
- Nothing may be **connected** to a ghost by hand, from either end: an edge out of it
  would claim the unknown system on its far side leads somewhere, which is the one thing
  nobody knows yet. The hole's own connection, made when it is raised, is the only one it
  has.
- **Resolving** a ghost sets its `solar_system_id`. If that system is *already* on the
  map (the hole led back into the chain), the ghost is **merged** instead: its
  connections move to the existing placement and the ghost row is deleted. The same
  path serves the manual "assign a system" action and the jump tracker, which discovers
  the same fact by flying it.
- A ghost lasts exactly as long as its scan says an unmapped hole is there. Deleting the
  signature or the system it hangs off cascades the node away; retyping the signature,
  linking it to a real system, or unlinking it leaves a node nothing claims, and that is
  swept up too. Every removal snapshots what it takes, so one undo brings the lot back. A
  **real** system left without connections stays: somebody put that on the map on purpose.
- The rule is re-established after **every** command rather than by the writes that
  happen to remember (`ghost::reconcile`), which is also how the `ghost_unlinked_wormholes`
  setting takes effect: turning it on draws the holes already scanned, turning it off takes
  those nodes away, leaving the scans alone. A consequence worth stating: cutting a ghost's
  edge, or unlinking its signature, does not make the hole go away — the scan still says it
  is there, so it is drawn again. Deleting the signature is how you say it was never found.
- The alias is **ephemeral**: removing the system from the map and re-adding it does
  **not** restore the previous alias.
- Removing a system deletes its placement, its **signatures**, and its connections —
  but **not** its persisted details (next table).

---

## `map_solar_system_details`

*Persisted intel.* Per-`(map, system)` intel that must **survive** the system being removed from the
map and be shown again when it is re-added. Lives independently of
`map_solar_systems`.

| Column             | Type        | Notes                                                    |
|--------------------|-------------|----------------------------------------------------------|
| `id`               | pk          |                                                          |
| `map_id`           | fk maps     |                                                          |
| `solar_system_id`  | int         | SDE `_key`                                               |
| `status`           | enum        | `unknown` (default) / `friendly` / `hostile` / `active` / `unscanned` / `empty` |
| `notes`             | text, null  | member-gated markdown notes (viewers never receive them)  |
| `occupying_group`  | text, null  | who holds the system (intel)                             |
| `updated_at`       | timestamptz |                                                          |

**Invariants & expected behaviour**

- Unique `(map_id, solar_system_id)`.
- A details row may exist with **no** corresponding `map_solar_systems` row (system
  not currently placed) — that is the whole point.
- Round-trip: add system → set status/occupying group → remove → re-add ⇒ the status
  and occupying group are still there. (Signatures, by contrast, are gone.)
- Deleting the **map** removes its details; removing a single **system** does not.

---

## `signatures`

A cosmic signature in a system, as scanned from the in-game probe scanner. For a
**wormhole** signature this is also where the hole's life-cycle state lives, because
that state is visible from the scan *before* the hole is jumped or connected. The
catalogue of signature groups and known wormhole types is
[custom static reference](./static.md).

| Column            | Type             | Notes                                                       |
|-------------------|------------------|-------------------------------------------------------------|
| `id`              | pk               |                                                             |
| `map_id`          | fk maps          |                                                             |
| `solar_system_id` | int              | SDE `_key` — the system the sig is in                       |
| `signature_id`    | text             | in-game id, e.g. `ABC-123`                                  |
| `group`           | enum             | `wormhole`, `data`, `relic`, `gas`, `combat`, `ore`, `homefront`, `faction_warfare`, `unknown` |
| `signature_type_id` | fk signature_types, null | the matched catalog type ([static reference](./static.md)) |
| `name`            | text, null       | raw scanner type name when no catalog type matched          |
| `size`            | enum, null       | `xl`, `large`, `medium`, `small` (wormholes)                |
| `mass_status`     | enum, null       | `stable`, `reduced`, `critical` ("massed")                  |
| `time_status`     | enum, null       | `stable`, `eol`, `critical` (≈ < 1h, "super EOL"), `expired` (closure imminent) |
| `time_status_updated_at` | timestamptz, null | when `time_status` last changed (trigger-maintained) |
| `connection_id`   | fk map_connections, null | set when this sig is linked as one end of a hole      |
| `created_at`      | timestamptz      |                                                             |
| `updated_at`      | timestamptz      |                                                             |

**Invariants & expected behaviour**

- Unique `(map_id, solar_system_id, signature_id)`.
- A `group = wormhole` signature **may carry `size` / `mass_status` / `time_status`
  with no `connection_id`** — the scanned-but-not-yet-jumped case. This is required.
- Only a `group = wormhole` signature may have a `connection_id`; the wormhole state
  columns are `null`/ignored for non-wormhole groups.
- **Ephemeral:** when a system is removed from the map, its signatures are deleted
  along with the placement (scan data goes stale; we do not persist it).
- `signature_id` is exactly 7 characters (`ABC-123`), enforced on add and paste.
- **Paste is upsert-only** (legacy semantics): it adds new sigs and refreshes existing
  ones (`group` = pasted else keep; an existing wormhole `signature_type_id` always
  survives; a site row takes the pasted catalog match or is cleared when only an
  unmatched raw name was pasted; recategorizing a hole to a site drops the link).
  It never deletes, and never touches `size` / `mass_status` / `time_status` /
  `created_at`. Rows missing from a scan are removed explicitly via the bulk delete.
- **Delete cascade** (legacy): deleting a signature also deletes its linked connection
  unless another signature *in the same system* still references it. The bulk delete
  additionally removes the far end of each deleted connection when it is not pinned /
  home / rally and has no remaining connections. The system the signatures were in
  always stays.
- **Expiry:** a background loop purges unlinked wormhole sigs older than 3 days and
  other sigs untouched for 7 days (a paste refreshes `updated_at`, keeping live sites
  alive). Linked sigs never expire.

### Keeping a connection and its signatures consistent

A jumped wormhole is described by up to **three rows for the same hole**: the
`map_connections` row plus its (≤2) signatures — one in each endpoint system. Their
shared state (`mass_status`, `time_status`, `size`) must agree. We call the connection
and its currently-linked signatures the **group**.

The state lives on **every** member (not just the signatures) because either can exist
alone: a connection can be marked massed/EOL before it's scanned, and a wormhole sig
carries state from the scanner before it's linked. So we can't designate one as the sole
source of truth. Instead a **PostgreSQL trigger keeps the whole group in lock-step**
(implemented in `migrations/0007_connection_sync.sql` and `0019_connection_size_lock.sql`;
the ordered enums mirror it in `src/maps/mod.rs`). A trigger rather than app-side code, so
the rows can never drift regardless of which path writes them, including the connection's
own `set_connection_status` and a signature's edit/link/paste.

Three rules:

- **Merge on link** — when a signature is linked to a connection (`connection_id` set),
  the group reconciles to the **worst (most-severe) non-null value per field**:
  `mass` `stable < reduced < critical`; `time` `stable < eol < critical < expired`; `size`
  `xl < large < medium < small` (smallest = most restrictive). A connection marked `eol`
  (<4h) linked to a sig scanned `critical` (<1h) becomes `critical`. Worst-wins is the
  safe, order-independent choice: the map never looks healthier than any observation of it.
- **Propagate on edit** — an explicit edit to any member (the connection or a linked sig)
  overwrites the whole group **verbatim**, so corrections and *downgrades* (e.g. back to
  `stable`) flow through. Because every merge/propagate equalises all three fields, a
  linked group is always fully consistent, so a single-field edit safely rewrites all
  three (the untouched two already match and the trigger's `IS DISTINCT FROM` guard — which
  is also what makes the cascade terminate — skips them).
- **Locked by type**: an identified wormhole type dictates `size`. Whenever any linked
  signature's catalog type carries a maximum jump mass (`wormhole_types.max_mass_per_jump`;
  K162 carries none), the group's `size` is the size that mass admits: `<= 5,000,000 kg`
  `small`, `<= 300,000,000` `medium`, `<= 1,000,000,000` `large`, else `xl` (the same
  thresholds as `sizeForJumpMass` client-side). The locked size wins over the merge and
  over any edit, so a manual size on the connection or a signature is overridden on the
  spot. Every write path passes through the same propagation (linking, editing, retyping a
  signature by hand or by paste, editing the connection), which is where the lock is
  applied (`map_locked_size`). Should two linked signatures both identify the hole with
  different masses, the most restrictive wins. Once no linked signature identifies the
  hole any more (retyped to K162, unlinked, deleted) the lock lifts and the group keeps the
  size it has.

Unlinking needs no sync: the remaining members keep their state, and the detached sig
keeps its last state as a standalone scanned wormhole. The relationship itself — at most
two signatures per connection, one per endpoint — is an invariant, not synchronized data.

## `map_watchlist`

Systems whose jump distance the navigation panel tracks (legacy
`map_route_solarsystems`). Map-scoped and shared; jump counts are computed
client-side, so the row is just membership plus the pin flag.

| Column | Type | Notes |
|---|---|---|
| `id` | pk | |
| `map_id` | fk maps | cascade |
| `solar_system_id` | fk solar_systems | unique per map |
| `is_pinned` | bool | pinned entries surface as route quick-picks |

Mutations are Member+, reads Viewer+; every change publishes
`MapEvent::WatchlistChanged`.

## `map_personal_watchlist`

One user's own watchlist on a map, seen by nobody else (after legacy PR #101). Same
columns as `map_watchlist` plus `user_id` (fk users, cascade), unique per map, user and
system. Any Viewer+ may keep one. Not a map command: no history, no undo, not exported;
a change only tells the user's own tabs, with `UserEvent::PersonalWatchlistChanged`.

---

## `map_ignored_solar_systems`

Systems the map as a whole keeps out of the way (legacy `map_ignored_solarsystems`):
the router steers around them for everyone, server-side searches (the Discord `route`
command, proximity and killmail alerts) never pass through them, and mapping as you fly
never places them. Map-scoped and shared, unlike the viewer's own route-around list,
which stays in the browser. The row is membership only.

| Column | Type | Notes |
|---|---|---|
| `id` | pk | |
| `map_id` | fk maps | cascade |
| `solar_system_id` | fk solar_systems | unique per map |

Mutations are Manager+ (the list changes what every pilot on the map sees and where
their jumps land), reads Viewer+; every change publishes
`MapEvent::IgnoredSystemsChanged`.

---

## `map_connection_jumps`

Every observed or manually logged transit through a wormhole connection, with the
ship's hull mass — the ledger behind the connection's mass-remaining estimate
(`jumps_count` / `jumps_mass_sum` aggregates on the connection payload).

| Column | Type | Notes |
|---|---|---|
| `id` | pk | |
| `map_id` | fk maps | cascade |
| `connection_id` | fk map_connections, null | null = pending (observed before the hole was mapped); cascade |
| `character_id` | fk characters, null | set null on character delete (the ledger survives) |
| `from_solar_system_id` / `to_solar_system_id` | fk solar_systems | transit direction |
| `ship_type_id` | fk types, null | set null |
| `ship_name` | text, null | |
| `mass` | bigint | hull mass in kg (`types.mass`); ±10% in game |
| `is_manual` | bool | manual entries; tracked jumps stay false even when corrected |

**Capture rules** (`src/maps/jumps.rs::record_transit`, called from the location
poller on every system change of a tracked character):

- Stargate pairs (per the `stargates` table) are never logged.
- Only maps where the character's user opted in (`map_user_settings.tracking_allowed`)
  and holds Member+ participate.
- A matching `wormhole` connection (either direction) claims the row immediately; a
  lone `stargate` edge means gate travel on that map → skip; no edge at all leaves a
  **pending** row (`connection_id` null) when the origin system is placed.
- A wormhole connection created within **120 s** claims matching pending rows;
  unclaimed rows are pruned after **10 minutes**. Claimed rows live until the
  connection (or map) dies, via cascade.
- The jump log endpoint returns the **latest 10** rows; the aggregates are full sums.
- The mass estimate is independent of the manual `mass_status` flag, and
  `preserve_mass` does not affect the math (display-only, matching legacy).

---

## `map_connections`

An edge between two placed systems. A wormhole edge is backed by the two signatures
above; a stargate edge is just a known gate.

| Column          | Type                 | Notes                                                    |
|-----------------|----------------------|----------------------------------------------------------|
| `id`            | pk                   |                                                          |
| `map_id`        | fk maps              |                                                          |
| `from_system`   | fk map_solar_systems |                                                          |
| `to_system`     | fk map_solar_systems |                                                          |
| `type`          | enum                 | `wormhole`, `stargate`                                   |
| `mass_status`   | enum, null           | `stable`, `reduced` ("massed"), `critical`               |
| `time_status`   | enum, null           | `stable`, `eol` (≈ <4h), `critical` (≈ <1h, "super EOL"), `expired` (closure imminent) |
| `size`          | enum, null           | `xl`, `large`, `medium`, `small` (max-jumpable class)    |
| `preserve_mass` | bool                 | exclude from mass bookkeeping (legacy flag; stored only) |
| `time_status_updated_at` | timestamptz, null | when `time_status` last changed (trigger-maintained) |
| `created_at`    | timestamptz          |                                                          |
| `updated_at`    | timestamptz          | bumped by edits and by the sync trigger                  |

**Invariants & expected behaviour**

- Connects two **distinct** systems in the **same** map (`from <> to`).
- The same pair may be connected **more than once** — parallel edges are allowed (two
  separate holes between the same systems), so there is no uniqueness constraint on
  `(map_id, from_system, to_system)`.
- The wormhole life-cycle state (`mass_status` / `time_status` / `size`) lives **both
  here and on the linked signatures**, kept in sync by a trigger (next section). It lives
  on the connection too because a connection can be marked massed/EOL **before** any
  signature is linked (a hole drawn before it's scanned); `null` means unknown. Ignored
  for `stargate` edges.
- A `wormhole` connection is backed by up to two signatures (one in `from_system`,
  one in `to_system`); a `stargate` connection has no signatures.
- Removing either endpoint system deletes the connection; deleting the connection
  clears `connection_id` on its signatures (they survive as plain wormhole sigs while
  their system is still placed).
