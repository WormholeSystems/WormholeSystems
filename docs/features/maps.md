# Map actions

The **business logic** for maps: the actions a user can take on a map and exactly how
each behaves. This is the application layer above the [database spec](../database/) —
it turns the tables ([mapping](../database/mapping.md), [access](../database/access.md))
into a set of authorized, validated operations.

Scope of this doc: **action-based, no UI.** Each action is a server-side function with a
clear signature, authorization rule, validation, effect, and error set — the contract
tests are derived from. UI / HTTP wiring (the Axum JSON routes in `src/api`) just
calls these.

In scope: map lifecycle (create / rename / delete / list / get), access grants, graph
editing (add / remove / move systems, add / remove connections, mark connection state),
and signatures (per-signature CRUD + linking a wormhole sig to a connection). **Out of
scope for now:** the scanner-**paste** reconciliation (bulk add/remove from a pasted scan
block — own future spec), live tracking, and routing.

Mutations also emit a realtime event so other viewers of the map refetch; that bus is the
in-process [`MapHub`](../features/realtime.md) and the `publish` calls are wired at the
(later) server-function layer — the action functions themselves stay pure.

## The actor

Every action is taken by a **user** acting **as one of their characters** (the
per-session active character — see [authentication](../database/authentication.md)).

```
Actor { user_id, character_id }
```

- `user_id` drives **authorization**: a user's effective role is the highest role they
  match across *all* their characters (a character's own id, its `corporation_id`, or
  its `alliance_id` matching a `map_access.subject_id`). See
  [access.md](../database/access.md).
- `character_id` is recorded where an action attributes ownership to a *character* (map
  creation grants ownership to the acting character, not the user).

`character_id` must belong to `user_id` — actions validate this and reject otherwise
(`Forbidden`).

## Action shape: command in, validation split

Every mutating action takes a **dedicated command struct** as its argument, with `actor`
as a *separate* parameter:

```
add_connection(pool, actor, AddConnection { map_id, from_system, to_system, kind })
```

- Named fields kill the "transposed positional argument" bug — several actions take
  multiple same-typed `i64`s (`map_id`, `from_system`, `to_system`) that are otherwise
  easy to swap.
- The command struct *is* the future HTTP/server-function request body (it derives
  `Serialize`/`Deserialize`). `actor` stays separate so it is always injected from the
  authenticated session — a client can never set its own `user_id`.

**Validation is split by what it needs:**

- **Pure / context-free** checks (non-blank name, `from != to`) live on the command as a
  `validate()` method the action calls first (only the few commands with real rules have
  one). They depend only on the input, so the **UI can reuse them** for pre-submit feedback.
- **Stateful / referential** checks (row exists, not already placed, both endpoints on
  *this* map, last-owner, character-belongs-to-user) **stay in the action**. They depend
  on DB state that can change between check and write, so they must be atomic with it;
  pulling them out would reintroduce a time-of-check/time-of-use race and let callers
  bypass invariants. Where the schema already enforces a rule (unique indexes, FKs, the
  `from <> to` CHECK), the action leans on it and maps the error to `MapError`.

So an action body reads: **validate input → authorize → perform (constraint-backed) → map
errors**, and stays authoritative for correctness.

## Roles

`Role` is an ordered enum, `Viewer < Member < Manager < Owner` (stored as text per the
[enum convention](../database/README.md)). An action states the **minimum** role it
requires; a user passes if their effective role is `>=` it.

| Action group        | Minimum role |
|---------------------|--------------|
| View / list / get   | Viewer       |
| Edit the graph (systems, connections) | Member |
| Manage access (grant / revoke / set role) | Manager |
| The map's ignore list (add / remove / clear) | Manager |
| Modify the map (rename, settings, delete) | Owner |

This mirrors the capability table in [access.md](../database/access.md#roles--capabilities).

### Effective-role resolution

`effective_role(map_id, user_id) -> Option<Role>`:

1. Collect the user's characters and each one's `(id, corporation_id, alliance_id)`.
2. Gather the EVE ids into one set.
3. Return the **max** `role` over the live `map_access` rows for this map whose
   `subject_id` is in that set, or `None` if no row matches.

The `map_user_roles` view does all three; `effective_role` reads one row of it.

`None` means "no access at all" and is reported as **`NotFound`** (we do not reveal that
a map the user can't see exists). Having *some* access but too low a role for the action
is **`Forbidden`**.

## Error model

A single `MapError` (thiserror) is returned by every action:

| Variant               | When                                                              |
|-----------------------|------------------------------------------------------------------|
| `NotFound`            | Map (or referenced row) doesn't exist, or the user has no access to it |
| `Forbidden`          | User has access but a lower role than the action requires; or acting as a character that isn't theirs |
| `Conflict`           | Uniqueness / idempotency violation (system already placed) |
| `Validation(String)` | Bad input (blank name, self-connection, endpoint not on the map, system id unknown) |
| `LastOwner`          | The operation would leave the map with zero owners                |
| `Db(sqlx::Error)`    | Underlying database error                                         |

Over HTTP, `Validation` and `Conflict` answer with their message alone (the status says
which it was), and `Db` is logged and answered with a generic message, since database
error text names tables and constraints.

## Transactions

Any action with more than one write runs in a single transaction, so it is all-or-
nothing: map creation (insert map **+** owner grant), system removal (the placement and
its cascaded connections/signatures), role changes that must re-check the owner
invariant. Single-statement actions don't need an explicit transaction.

---

## Map lifecycle

### `create_map(actor, name, description?) -> Map`

- **Auth:** any authenticated user, unless `MAP_CREATOR_AFFILIATION_IDS` is set: then the
  acting character, its corporation or its alliance has to be on that list, or it is
  `Forbidden`. `may_create_map` answers the same question for `/api/me`'s
  `can_create_map`, and `import_map_as_new` applies the same rule.
- **Validates:** `name` is trimmed and non-empty; `character_id` belongs to `user_id`.
- **Effect (one transaction):** insert the `maps` row, then insert a `map_access` row
  `(subject_type = character, subject_id = actor.character_id, role = owner)`.
- **Invariants:**
  - The new map has exactly one access row, an `owner` for the acting character.
  - A blank/whitespace name is rejected with `Validation`.

### `update_map(actor, map_id, { name?, description?, image_url? }) -> Map`

- **Auth:** `Owner`.
- **Validates:** if `name` is given it must be non-empty after trimming. Omitted fields
  are left unchanged; `description` / `image_url` can be explicitly set to `null`.
- **Effect:** updates the `maps` row.
- **Note:** swapping `image_url` does not touch object storage here; the old image is
  cleaned up by the app outside this action (see
  [maps invariants](../database/mapping.md#maps)).

### `delete_map(actor, map_id)`

- **Auth:** `Owner`.
- **Effect:** deletes the `maps` row; the DB cascades placements, details, connections,
  signatures, and access grants. Object-storage image cleanup is app-level, outside this
  transaction.
- **Invariant:** after deletion no rows in any child table reference `map_id`.

### `list_maps(user_id) -> Vec<(Map, Role)>`

- **Auth:** any authenticated user; returns only maps the user can access.
- **Effect:** read-only. Each map is paired with the user's effective role on it.
- **Invariant:** a map appears iff `effective_role` is `Some`.

### `get_map(actor, map_id) -> MapView`

- **Auth:** `Viewer`.
- **Effect:** read-only; returns the map plus its placed systems and connections (the
  graph). Live pilot locations are **not** included here (that's the member-gated
  tracking path).

---

## Access management

### `set_access(actor, map_id, subject_type, subject_id, role)`

- **Auth:** `Manager`.
- **Privilege ceiling:** an actor may not grant a role **higher than their own**
  effective role. So a `Manager` can grant up to `Manager`; only an `Owner` can grant
  `Owner`. Violation → `Forbidden`.
- **Effect:** upsert the grant for `(map_id, subject_id)` (unique per
  [access.md](../database/access.md#map_access)); an existing subject's role is changed
  in place.
- **Invariants:**
  - `subject_id` is unique per map; re-granting the same subject updates its role.
  - Downgrading or changing the **last remaining owner** such that no owner is left is
    rejected with `LastOwner`.

### `revoke_access(actor, map_id, subject_id)`

- **Auth:** `Manager`.
- **Effect:** delete the grant.
- **Invariant:** revoking the **last owner** is rejected with `LastOwner` — every map
  always has at least one owner.

> **Open — owner constraints** (carried from [access.md](../database/access.md)): should
> `owner` / `manager` be restricted to `subject_type = character`? Single owner per map,
> or co-owners allowed? This spec assumes **multiple owners allowed, any subject type**,
> enforcing only "≥ 1 owner". Tightening later is additive.

---

## Graph editing

### `add_system(actor, map_id, solar_system_id, x, y, alias?) -> MapSolarSystem`

- **Auth:** `Member`.
- **Validates:** `solar_system_id` exists in [`solar_systems`](../database/universe.md);
  the system is not already placed on this map.
- **Effect:** insert a `map_solar_systems` row at `(x, y)`.
- **Invariants:**
  - Unique `(map_id, solar_system_id)` — a second add of the same system → `Conflict`.
  - Adding a system **does not** create or reset its
    [`map_solar_system_details`](../database/mapping.md#map_solar_system_details); if a
    details row already exists (system was placed before), it is left intact and resurfaces.
  - An unknown `solar_system_id` → `Validation`.

### Ghost placements (raised by the signature writes)

Not an action of its own: a hole exists because a scan says so, so
`paste_signatures`, `add_signature` and `update_signature` raise the nodes themselves,
inside the same transaction.

- **When:** the map has `ghost_unlinked_wormholes` set, and the write leaves a
  `wormhole` signature in that system with no connection.
- **Effect:** per such signature, a `map_solar_systems` row with **no** `solar_system_id`
  (a [ghost](../database/mapping.md#ghost-placements)), a wormhole connection from the
  scanned system to it, and the signature linked to that connection. The spot is the
  first free slot beside the scanned system, then down that column.
- **Invariants:**
  - A signature already carrying a connection is skipped, so one hole is one node and a
    repeated paste adds nothing.
  - The scan and the nodes it raised are **one** entry in the history: undo takes them
    back together.
  - The setting is enforced here rather than by the client, so it holds whoever wrote.

### `resolve_ghost_system(actor, map_id, map_solar_system_id, solar_system_id?) -> MapSolarSystem`

- **Auth:** `Member`.
- **Validates:** the placement is a ghost; the system exists; the hole does not lead back
  to the system it hangs off.
- **Effect:** set the ghost's `solar_system_id`, keeping its position, alias and edges. If
  that system is already on the map, **merge** instead: its connections move to the
  existing placement and the ghost row is deleted. Transits recorded before the hole was
  named are claimed by its connections either way.
- **Invariants:**
  - Resolving a real placement → `Validation`; the map never re-points a system.
  - `solar_system_id: None` turns it back into a ghost, and is refused once the system
    holds signatures. It exists as the inverse of a resolve rather than as an action.

### Cleaning the map

Not a command of its own: "clean map" is `remove_systems` over the systems no **anchor**
reaches any more, worked out on the client so the same list can be shown before it runs.
An anchor is a pinned system or the map's home system. A map with no anchors has nothing
adrift rather than everything, since there would be nothing to measure reachability from.

The placement mode itself is chosen in the map's setup walkthrough (Manager+ only) or its
settings, not at creation: a new map has nothing on it to picture either way round.

Distinct from `clear_map`, which takes everything *except* the anchors. Clean takes the
dead branches a collapsed hole left behind; clear empties the map back to its anchors.

### `remove_system(actor, map_id, map_solar_system_id)`

- **Auth:** `Member`.
- **Effect:** delete the `map_solar_systems` row. The DB cascades the system's
  **signatures** and any **connections** it is an endpoint of, and the command takes the
  signatures on the *other* side of those connections too: a signature is the record of a
  hole, so removing where the hole led removes the hole. Its **details persist**.
- **Invariants:**
  - The placement, its signatures, and its connections are gone.
  - Its `map_solar_system_details` row (if any) still exists — round-trips on re-add.
  - Removing a system on a different map, or an id not on this map → `NotFound`.

### `move_system(actor, map_id, map_solar_system_id, x, y)`

- **Auth:** `Member`.
- **Effect:** update the placement's `(position_x, position_y)`.

### `set_alias(actor, map_id, map_solar_system_id, alias?)`

- **Auth:** `Member`.
- **Effect:** set or clear the **ephemeral** alias on the placement.
- **Invariant:** the alias is not persisted across removal (it lives on
  `map_solar_systems`, which is deleted on remove).

### `add_connection(actor, map_id, from_mss_id, to_mss_id, type) -> MapConnection`

`from_mss_id` / `to_mss_id` are `map_solar_systems` ids (placements), not SDE system ids.

- **Auth:** `Member`.
- **Validates:**
  - both placements exist and belong to **this** map;
  - they are **distinct** (`from <> to`);
  - `type` is `wormhole` or `stargate`.
- **Effect:** insert a `map_connections` row.
- **Invariants:**
  - A self-connection → `Validation`; an endpoint not on the map → `Validation`.
  - The same pair may be connected **more than once** — parallel edges are allowed (e.g.
    two distinct wormholes between the same systems), so a repeat is *not* a conflict.
  - A new connection starts with **unknown** life-cycle state (all null).

### `set_connection_status(actor, map_id, connection_id, { mass_status?, time_status?, size? }) -> MapConnection`

Mark a connection's wormhole life-cycle state — works whether or not a signature is linked.

- **Auth:** `Member`.
- **Validates:** the connection exists on this map (else `NotFound`). Partial update:
  `None` leaves a field unchanged, `Some(None)` clears it to unknown, `Some(Some(v))` sets it.
- **Effect:** update the row's `mass_status` / `time_status` / `size`. If a signature is
  linked, the DB trigger **propagates the new state to it (and its sibling) verbatim** — see
  [the sync spec](../database/mapping.md#keeping-a-connection-and-its-signatures-consistent).
- **Invariant:** state set here is visible on every group member; downgrades propagate.

### Connection jumps (`add_jump` / `update_jump` / `remove_jump` / `list_jumps`)

- **Auth:** mutations `Member`, list `Viewer`.
- `add_jump(map_id, connection_id, direction, { ship_type_id?, mass? })`: manual log
  entry; needs a mass or a ship type (hull mass derived); `direction`
  outbound/inbound resolves the endpoints from the connection; `is_manual = true`.
- `update_jump`: partial edit; a changed ship type without an explicit mass
  re-derives the hull mass; `is_manual` never flips.
- `list_jumps`: the latest 10, newest first (aggregates ride on the connection).
- Automatic capture and claiming/pruning are documented in
  [the table spec](../database/mapping.md#map_connection_jumps).

### `remove_connection(actor, map_id, connection_id)`

- **Auth:** `Member`.
- **Effect:** delete the `map_connections` row. Linked signatures survive with their
  `connection_id` cleared (DB `on delete set null`).
- **Invariant:** a connection id not on this map → `NotFound`.

### `add_eve_scout_connections(actor, map_id, hub_solar_system_id, at?) -> count`

Put EVE Scout's public holes out of a hub (Thera or Turnur) on the map in one step. The
API fetches the current EVE Scout list (`GET /api/evescout`, see [Navigation](#navigation))
and hands the hub's rows to the command; the command never talks to EVE Scout itself.

- **Auth:** `Member`.
- **Validates:** `hub_solar_system_id` is Thera or Turnur (else `Validation`). A row whose
  far side is not in `solar_systems` is skipped, not fatal.
- **Effect, in order, all idempotent:**
  1. Place the hub if it is not on the map: at `at` when given, else at the map's
     default spot, moved to the first free slot either way.
  2. For each hole, sorted known space by security (highest first) then wormholes by
     class: place the far system if it is not on the map, at the first free slot beside
     the hub; siblings stack down the column.
  3. Connect the hub and the far system unless any connection already joins the two
     placements. A new one is a `wormhole` sized from EVE Scout's `max_ship_size`, then
     given EVE Scout's mass and lifetime state. An existing one is left alone.
  4. On each end, the signature EVE Scout names (a 7-character id; anything else is
     skipped): a missing one is added as a `wormhole` with the identified type (the
     `signature_types` row whose code is EVE Scout's `wh_type`, on both ends as the
     legacy did) and the hole's state, then linked. An existing `unknown` or unlinked
     `wormhole` signature is promoted, given the type when it has none, and linked; the
     link's merge reconciles its state. A signature that is linked already, or scanned
     as another group, is left as it is.
- **Returns:** how many connections were added.
- **Undo:** one step. Signatures that were promoted or linked go back to how they were
  scanned; everything the command added (placements, connections, signatures) is
  removed. A run that changed nothing records an audit row with no undo.
- **Events:** `SystemAdded` per placement, `ConnectionChanged` per new connection,
  `SignatureChanged` per system whose signatures changed.

---

## Signatures

Cosmic signatures scanned in a placed system. A `wormhole`-group signature carries the
hole's `mass_status` / `time_status` / `size` from the scanner and can be **linked** to a
connection as one of its ends; non-wormhole groups carry none of that. A signature may
reference a catalog type (`signature_type_id`); `name` holds the raw scanner type name
when nothing matched.

### `add_signature(actor, map_id, solar_system_id, signature_id, group, { signature_type_id?, name?, size?, mass_status?, time_status? }) -> Signature`

- **Auth:** `Member`.
- **Validates:** `signature_id` is 7 chars; the system is **placed** on this map (else
  `Validation`); only a `wormhole` group may carry `size` / `mass` / `time` (else
  `Validation`); a catalog type must belong to the group's category.
- **Effect:** insert a `signatures` row (unlinked — `connection_id` null).
- **Invariants:** duplicate `signature_id` in the same `(map, system)` → `Conflict`.

### `update_signature(actor, map_id, signature_pk, { group?, signature_type_id?, name?, size?, mass_status?, time_status? }) -> Signature`

- **Auth:** `Member`.
- **Validates:** wormhole-only rule for the state fields; a catalog type must belong to
  the (new) group's category. Partial-update semantics as above.
- **Effect:** update the row. **Changing the group clears the catalog type, the
  connection link, and any wormhole state** (the legacy category-select behavior),
  unless a new type comes in the same call. If linked, a state edit **propagates to the
  connection** (and sibling) verbatim via the trigger.

### `link_signature(actor, map_id, signature_pk, connection_id) -> Signature`

- **Auth:** `Member`.
- **Validates:** the signature is a `wormhole` (else `Validation`); the connection is on
  this map and **reaches the signature's system** (one of its endpoints is in that system).
- **Effect:** set `connection_id`, which fires the **merge** — the group reconciles to the
  worst state per field. Returns the signature *after* the merge.

### `unlink_signature(actor, map_id, signature_pk) -> Signature`

- **Auth:** `Member`.
- **Effect:** clear `connection_id`. The signature keeps its last state as a standalone
  scanned wormhole; the connection and any sibling are untouched.

### `remove_signature(actor, map_id, signature_pk)`

- **Auth:** `Member`. **Effect:** delete the row. Legacy cascade: if it was the last
  signature on its side of a linked connection, the connection is deleted too (the other
  end's signature is unlinked by the FK). Unknown id → `NotFound`.

### `remove_signatures(actor, map_id, signature_pks)`

- **Auth:** `Member`. The panel's "delete missing signatures" path.
- **Effect:** bulk delete with the full cascade: each linked connection whose side is
  left without signatures is deleted, and its far end is removed from the map when it
  is not pinned / home / rally and has no remaining connections. The system the
  signatures were in always stays, even once it has lost its last connection.

### `paste_signatures(actor, map_id, solar_system_id, signatures)`

- **Auth:** `Member`. Rows are `{signature_id, group?, signature_type_id?, name?}`, parsed
  by the client (`parseScan`), which maps localized group and site names from every EVE
  client language to their English catalog names before matching.
- **Validates:** the system is placed; every `signature_id` is 7 chars.
- **Effect:** upsert-only reconciliation (see the
  [invariants](../database/mapping.md#signatures)): adds new rows, refreshes existing
  ones, preserves wormhole types and links, never deletes, never touches life-cycle
  state. Deleting vanished rows is the client's explicit `remove_signatures` call. Sets
  the placement's `signatures_pasted_at` to now (even when nothing in the list changed),
  which the signatures panel shows as the list's age; undo restores the previous time.
- **Events:** `SignatureChanged` for the system and `SystemDetailsChanged` for its
  placement.

### `list_signatures(actor, map_id) -> Vec<Signature>`

- **Auth:** `Viewer`. **Effect:** read-only; every signature on the map.

---

## Per-viewer background image

`PUT /api/maps/{id}/background-image` (multipart, one `image` part), `GET` the same path
to fetch it, `DELETE` to take it away. All three need viewer access; the file is the
caller's own, so no role above that is involved and nobody else's view changes.

- The upload replaces whatever was there and answers with the caller's full settings, the
  same shape `GET /api/maps/{id}/settings/user` gives, now carrying
  `background_image_url`. That URL points at the `GET` and changes with every upload.
- Validation is on the bytes: PNG, JPEG, GIF or WebP by signature, 8 MiB at most. A part
  that is not an image, or is too large, is a 400 with the reason; a body over the limit
  is refused by the server before the handler runs.
- The mode (`grid` or `viewport`) is an ordinary field of the settings update.

## Testing approach

Every action has DB-backed tests using `#[sqlx::test]`, which provisions an **isolated
database per test** and runs the migrations. Each test seeds only the minimal fixtures
it needs — a user, one or more characters with affiliations, and a handful of
`solar_systems` rows — then drives actions and asserts on both the return value and the
resulting rows.

The tests are split by area, each its own integration-test binary, with shared fixtures
in `tests/common/`:

- `tests/maps_map.rs` — create / update / delete / list / get
- `tests/maps_access.rs` — grant / change role / revoke + the owner invariant
- `tests/maps_solar_system.rs` — placing / moving / removing / aliasing systems
- `tests/maps_connection.rs` — connecting / disconnecting + marking connection state
- `tests/maps_signatures.rs` — signature CRUD, linking, and the connection↔signature sync

Coverage targets, per action: the **happy path** asserting the exact effect (return-value
fields *and* the resulting rows match the inputs), each **authorization** boundary (role
just-too-low rejected, minimum role accepted), and each **invariant/error** bullet above
(e.g. duplicate add → `Conflict`, last-owner revoke → `LastOwner`, system round-trip
preserves details). The bullet lists in this doc are the test checklist.

---

## Navigation

The navigation panel is client-routed (Dijkstra over the stargate graph plus the
map's live wormhole edges, and optionally EVE Scout's public Thera/Turnur edges from
`GET /api/evescout`, proxied + cached 60s). One origin drives everything: the route
"From" when set, else the active system, else the tracked character's location.

- **Watchlist** (`map_watchlist`): `list/add/set_pinned/remove` (Member+ mutations,
  Viewer+ reads). Jump counts, colors, and route popovers are client-side.
- **Personal watchlist** (`map_personal_watchlist`): the same verbs under
  `/watchlist/personal`, for the caller alone, Viewer+. Its own tab in the panel.
- **Find** measures from the same origin; its tab shows only the origin picker.
- **Route settings** live on `map_user_settings` per user per map:
  `route_preference` (`shorter`/`safer`/`less_secure`), `security_penalty` (0-100),
  `route_allow_time_status` (`stable`/`eol`/`critical`/`expired` = worst lifetime still
  traversed; the default `critical` keeps expired holes out), `route_allow_mass_status` (`stable`/`reduced`/`critical`), and
  `route_use_evescout`.
- Two ignore lists. The viewer's own route-around list is client-side per map,
  deliberately not server state: it is a scratch decision for one trip. The map's
  ignore list (`map_ignored_solar_systems`) is shared, see below.
- EVE Scout's holes can also be put on the map, one hub at a time:
  [`add_eve_scout_connections`](#add_eve_scout_connectionsactor-map_id-hub_solar_system_id-at---count).
- `GET /api/routing-graph` also carries `jove` and `stations` system-id sets for the
  Find (closest systems) conditions.

## Ignored systems

`map_ignored_solar_systems` is the map-wide list of systems everyone steers around.
Three things read it:

- the client router leaves them out of every route, jump count and Find result, on top
  of the viewer's own list;
- the server searches (`alerts::proximity::nearest`, so the Discord `route` command and
  the proximity / killmail alerts) never traverse one, except as the origin or the target
  of the search itself;
- `track_jump` refuses to place one: a tracked pilot flying through a trade hub does not
  put the hub on the chain. A system already on the map stays reachable.

### `list_ignored_systems(actor, map_id) -> Vec<IgnoredSystem>`

Viewer+. Share-link watchers read it through the same unauthenticated path as the
watchlist.

### `add_ignored_system(actor, map_id, solar_system_id) -> IgnoredSystem`

Manager+. Idempotent per map and system; an unknown system is `Validation`. Inverse is
`remove_ignored_system`.

### `remove_ignored_system(actor, map_id, solar_system_id)`

Manager+. A system not on the list is `NotFound`. Inverse is `add_ignored_system`.

### `clear_ignored_systems(actor, map_id) -> count`

Manager+. Inverse re-adds every row it removed, as one history step.

All three publish `MapEvent::IgnoredSystemsChanged { map_id }`. Import and export carry
the list in the `routes.ignored_solarsystems` section of the legacy file format.
