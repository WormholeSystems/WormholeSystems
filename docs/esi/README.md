# ESI (EVE Swagger Interface)

The endpoints `wormholesystems` consumes from CCP's REST API — one file per endpoint, with
parameters, the exact response structure, and examples. Curated, not a mirror of the
full [openapi.json](https://esi.evetech.net/meta/openapi.json).

- **Base URL:** `https://esi.evetech.net`
- **Compatibility date:** pinned to **`2026-06-09`**, sent as the `X-Compatibility-Date`
  header (or `?compatibility_date=`). Bump deliberately and re-check affected endpoints.
- **Auth:** *authenticated* endpoints take a Bearer access token from
  [SSO](../database/authentication.md) for a character that granted the listed scope;
  *public* endpoints take none.
- **Polling:** ESI has no push. Respect `ETag` / `If-None-Match` and cache-expiry
  headers, and back off on the ESI error-limit headers. What the client does:
  - The three presence routes (location, ship, online) keep their last reply per
    character. Inside its cache window (`Cache-Control: max-age`, else `Expires` against
    `Date`) the stored reply is used without asking; after it the request carries the
    `ETag`, and a `304` reuses the stored body at half the rate-limit cost.
  - A `420`, or fewer than 10 errors left in the error window
    (`X-ESI-Error-Limit-Remain`), stops every request until
    `X-ESI-Error-Limit-Reset`. A `429` stops requests with that token (or every public
    request) for its `Retry-After`. Either way the request fails at once without being
    sent, and the pollers pick it up on a later tick.
- Common headers (`X-Compatibility-Date`, `If-None-Match`, `If-Modified-Since`,
  `Accept-Language`) are omitted from each endpoint's parameter table — they apply
  everywhere.
- Each endpoint file links to its human-readable
  [API Explorer](https://developers.eveonline.com/api-explorer) page.

## Endpoints

**Character presence** — authenticated; feeds
[`character_status`](../database/tracking.md#character_status):

- [Get character location](./character-location.md) — `esi-location.read_location.v1`
- [Get current ship](./character-ship.md) — `esi-location.read_ship_type.v1`
- [Get character online](./character-online.md) — `esi-location.read_online.v1`

**UI** — authenticated:

- [Set autopilot waypoint](./ui-autopilot-waypoint.md) — `esi-ui.write_waypoint.v1`

**Public lookups** — no token; resolve names and keep access-driving affiliations
fresh:

- [Character public info](./character-public.md)
- [Character affiliation (bulk)](./characters-affiliation.md)
- [Corporation public info](./corporation-public.md)
- [Alliance public info](./alliance-public.md)

**Universe** — public:

- [List sovereignty of systems](./sovereignty-systems.md) — feeds
  [`system_sovereignty`](../database/universe.md#system_sovereignty)

## Scopes we request

These map one-to-one to rows in the [`esi_scopes`](../database/authentication.md#esi_scopes)
catalogue and are what we ask for at SSO consent:

- `esi-location.read_location.v1` — character system / docked location
- `esi-location.read_ship_type.v1` — current ship
- `esi-location.read_online.v1` — online state
- `esi-ui.write_waypoint.v1` — set autopilot waypoint

The public lookups need **no** scope.
