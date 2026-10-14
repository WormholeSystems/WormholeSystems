# Character Tracking

Live, ESI-sourced presence for a character — where they are, whether they're online,
and what ship they're in. This is the data behind the **viewer-vs-member** line:
[members and above](./access.md#roles--capabilities) see it on the map; viewers do
not. Part of the [database spec](./README.md) — see it for conventions and goals.

## `character_status`

The latest known status of a character — one row per character (1:1), refreshed by a
tiered [polling job](../processes.md#character-status-polling) against
[ESI](../esi/README.md), within the character's
[granted scopes](./authentication.md#esi_scopes). (A 1:1 current-state table, so the
singular name reads better than the usual plural.)

| Column            | Type              | Notes                                                |
|-------------------|-------------------|------------------------------------------------------|
| `character_id`    | pk, fk characters | 1:1 with the character                               |
| `solar_system_id` | int, null         | → [`solar_systems`](./universe.md#solar_systems)     |
| `station_id`      | bigint, null      | docked NPC [`station`](./universe.md#stations), if any |
| `structure_id`    | bigint, null      | docked [`structure`](./universe.md#structures), if any |
| `is_docked`       | bool, generated   | `station_id IS NOT NULL OR structure_id IS NOT NULL` |
| `online`          | bool              | currently online                                     |
| `last_online_at`  | timestamptz, null | when last seen online, to within a tier-1 tick (persists while offline) |
| `ship_type_id`    | int, null         | current ship hull → [`types`](./types.md)            |
| `ship_name`       | text, null        | player-given ship name                               |
| `ship_item_id`    | bigint, null      | ship instance id; a change signals a ship swap       |
| `ship_updated_at` | timestamptz, null | when the current ship was first observed             |
| `updated_at`      | timestamptz       | when a polled value last changed                     |

**Invariants & expected behaviour**

- One row per character; deleting the character deletes its status.
- `is_docked` is **derived** (a Postgres generated column): the character is docked
  iff a station or structure id is present, otherwise in space. No separate flag to
  drift — same single-source-of-truth principle used elsewhere in the spec.
- Tracking depends on **scopes**: location, online, and ship each need the matching
  ESI scope granted (see [`esi_scopes`](./authentication.md#esi_scopes) /
  [`esi_token_scopes`](./authentication.md#esi_token_scopes)). A field whose scope is missing
  stays `null` rather than guessed.
- `last_online_at` persists while the character is offline, so the UI can show
  "last seen N ago".
- A change in `ship_item_id` marks a ship swap and bumps `ship_updated_at`.
- A poll that sees nothing new writes nothing: each write is conditional on a value
  actually differing, so the row (and `updated_at`) only moves on a change.
- **Visibility is permission-gated:** status is shown only to map users with the
  **member** role or higher (see [Access](./access.md#roles--capabilities)); viewers
  never receive it. Status is global to the character — it surfaces on any map that
  displays the character's current system, to permitted users.

> **Open — history.** Cadence is specified in
> [character status polling](../processes.md#character-status-polling). Still open: keep
> only the current snapshot (modelled here) or a movement history for trails / replay?

## `map_user_settings`

Per-user, per-map preferences.

| Column              | Type        | Notes                                              |
|---------------------|-------------|----------------------------------------------------|
| `map_id`            | pk part, fk maps     | cascade on map delete                     |
| `user_id`           | pk part, fk users    | cascade on user delete                    |
| `tracking_allowed`  | bool, default false  | explicit opt-in to share the user's characters' live location on this map |
| `show_threat_level` | bool, default true   | whether threat rings render for this user |
| `follow_character`  | bool, default false  | select the system a tracked pilot is in as they fly, so the side cards keep up |
| `prompt_for_signature` | bool, default true | ask which signature was jumped on arriving somewhere new, rather than mapping the hole unlinked |
| `preselect_signature` | bool, default false | open that prompt with the likeliest signature already chosen, so scanner-order jumps are one Enter each. Only read while `prompt_for_signature` is on |
| `tracked_character_ids` | bigint[], default empty | the user's pilots whose jumps build this map; empty means the character the session acts as. Chosen per map, so a farm alt maps the farm chain and never the main map |
| `background_image_path` | text, nullable | the file this user put behind the canvas on this map, relative to the uploads directory. Null means the plain grid |
| `background_image_mode` | enum `map_background_mode`, default `grid` | `grid` paints the image across the whole world so it pans and zooms with the systems; `viewport` pins it to the visible panel |
| `updated_at`        | timestamptz          |                                           |

### Background image

A viewer may put a picture behind their own view of the map: a corp logo, a chain
diagram, anything. It is theirs alone, like every other row in this table, so two people
on the same chain can look at different backgrounds.

- The file lives on the API server's disk under `UPLOADS_DIR` (default `data/uploads`),
  at `map-backgrounds/{map_id}/{user_id}-{stamp}.{ext}`; the row keeps that relative
  path. The stamp makes every upload a new URL, so a browser never shows a stale cached
  image after a replacement.
- PNG, JPEG, GIF and WebP only, by the bytes rather than the declared type, and at most
  8 MiB. Anything else is refused before it touches the disk.
- Replacing or removing the image deletes the old file. Deleting the map or the user
  cascades the row away but leaves the file; it is cleaned up by hand, and is bounded by
  8 MiB per viewer per map.
- The image is served back only to its owner, and only while they can still view the map.
  Guests on a share link have no settings row and see the plain grid.

Presence (`GET /api/maps/{id}/characters`) shows a character only when: its user opted in
here, the character holds a location-scoped ESI token, it is online, and the user has
member-or-better access. Viewers can neither appear nor see presence.
