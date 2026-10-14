# Access

Who can see and do what on a map: the `map_access` grants and the role / capability
model. Part of the [database spec](./README.md). Subjects reference EVE characters,
corporations, and alliances (see [Authentication](./authentication.md)) and grant a
role on a [map](./mapping.md#maps).

## `map_access`

Grants access to a map for a mix of characters, corporations, and alliances — each at
a role. The **owner** is just a `role = owner` row here; there is no `owner_id`.

| Column         | Type        | Notes                                         |
|----------------|-------------|-----------------------------------------------|
| `id`           | pk          |                                               |
| `map_id`       | fk maps     |                                               |
| `subject_type` | enum        | `character`, `corporation`, `alliance`        |
| `subject_id`   | bigint      | EVE id of the character/corp/alliance         |
| `role`         | enum        | `viewer`, `member`, `manager`, `owner`        |
| `created_at`   | timestamptz |                                               |

**Invariants & expected behaviour**

- **Shared maps are read-only.** `maps.is_public` opens a map to anyone at all;
  `maps.share_token` opens it to whoever holds the token. Either way the visitor resolves
  to `viewer` and may have no account, so nothing that writes and nothing Member+ (pilots,
  history) is reachable. The token is never sent to anyone below manager, since it is a key
  to the map. Minting a new token replaces the old one, which is how a link is withdrawn.
- **A share link is a way in, not a place.** `/share/{token}` resolves the token, leaves it
  in a `map_share_{id}` cookie and redirects to `/maps/{id}`: a watcher reads the same page
  everybody else does, with the parts that need an account absent. Read handlers take the
  token from the query string or that cookie, so the address bar goes back to being an
  ordinary map link once the visitor is in.
- **A map has exactly one owner.** `owner` cannot be granted through `set_access`; it
  moves with `transfer_ownership`, which demotes the previous owner to `manager` and
  requires the new one to be a character already granted access. Creating a map makes its
  creator the owner.
- `expires_at` (nullable) ends a grant on its own. `null` is the ordinary grant, which
  lasts until it is revoked. A lapsed row is left in place but counts for nothing: it is
  excluded from every role lookup and from the map's own list of grants, so re-granting
  the same subject simply revives it.

- A map can hold **any mix** of subject types at once (e.g. one character + two
  corporations + one alliance), each with its own role.
- **Unique `(map_id, subject_id)`.** EVE entity ids are globally unique across
  characters, corporations, and alliances, so `subject_id` alone is unique per map;
  `subject_type` is descriptive. A given subject therefore holds exactly one role on
  a map.
- A map always has **at least one** `role = owner` row (this is the "every map has an
  owner" invariant; see [`maps`](./mapping.md#maps)).
- **Access check:** a user may access a map if, for any of their characters, that
  character's id, `corporation_id`, or `alliance_id` matches a `map_access` row's
  `subject_id`. The match is uniform — owner is no longer a special case.
- The user's **effective role** is the highest they match across all their characters
  (owner > manager > member > viewer).
- Both are views, so no query spells them out again: `user_subjects (user_id,
  subject_id)` is every id a user's grants can name, and `map_user_roles (map_id,
  user_id, role)` is their effective role on each map, over live grants only
  (`map_access_live`). Migration 0037.
- Because a grant can target a corporation or alliance, the check is only as correct as
  each character's stored affiliation. That freshness is maintained by
  [affiliation refresh](../processes.md#affiliation-refresh) (on login + periodic), so
  a character that leaves a granted corp/alliance loses that access within one refresh
  interval.

## Roles & capabilities

| Capability                                                        | Viewer | Member | Manager | Owner |
|-------------------------------------------------------------------|:------:|:------:|:-------:|:-----:|
| View systems, connections, signatures                             |   ✅   |   ✅   |   ✅    |  ✅   |
| See **live** member locations (where pilots are)                  |   ❌   |   ✅   |   ✅    |  ✅   |
| Edit the graph: move systems, add/remove systems & connections, edit sig/connection state | ❌ | ✅ | ✅ | ✅ |
| Manage access: add/remove people from the map                     |   ❌   |   ❌   |   ✅    |  ✅   |
| Modify the map itself: rename, settings, **delete**               |   ❌   |   ❌   |   ❌    |  ✅   |

The viewer-vs-member line is specifically: a **viewer sees the static map** (systems,
connections, signatures) but **not live pilot locations**; a **member** sees live
locations and can edit the graph. Live locations come from
[`character_status`](./tracking.md) and depend on the character having granted the
relevant ESI scopes (see [Authentication](./authentication.md)).

> **Open — owner constraints.** Two small follow-ups now that owner is a `map_access`
> role: (a) should an `owner` (and probably `manager`) grant be restricted to
> `subject_type = character`, since granting ownership to a whole corp/alliance is
> rarely intended? (b) Exactly one owner per map, or allow co-owners? "At least one"
> is already required; cap at one with a partial unique index if single-owner.
