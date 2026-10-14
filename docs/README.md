# WormholeSystems — documentation

`wormholesystems` is a Rust rewrite of [**WormholeSystems**](https://wormhole.systems/), a
real-time, collaborative wormhole-mapping tool for EVE Online.

It lets a corp or alliance map a wormhole chain together: pilots share one live map,
paste probe-scanner results to sync signatures, track the mass and lifetime state of
each connection, see where each other are via ESI, and find the shortest way home —
with layered access control and Discord alerting.

## Contents

- [`database/`](./database/) — the application data model: goals, tables,
  relationships, and expected behaviour, split by domain
  ([mapping](./database/mapping.md), [authentication](./database/authentication.md),
  [access](./database/access.md), [tracking](./database/tracking.md),
  [universe](./database/universe.md), [item types](./database/types.md),
  [custom static reference](./database/static.md)).
- [`features/`](./features/) — application behaviour above the data model: the
  authorized, validated actions users take. ([map actions](./features/maps.md) —
  maps, access, graph editing, connections, and signatures; [realtime](./features/realtime.md)
  — the in-process event bus that pushes map changes to viewers.)
- [`esi/`](./esi/) — the EVE ESI endpoints we consume: parameters, response
  structure, and examples, one file per endpoint.
- [`api.md`](./api.md) — the API: the generated OpenAPI document, tokens, errors and
  conventions; [`realtime-api.md`](./realtime-api.md) — the WebSocket streams the
  document cannot describe.
- [`processes.md`](./processes.md) — background / scheduled work (e.g. affiliation
  refresh) and the data it touches.
- [`ui-style-guide.md`](./ui-style-guide.md) — the interface design language: slim,
  minimal, monochrome, theme tokens, and which component libraries to use.
- [`deployment.md`](./deployment.md) — standing a deployment up with `wsctl`, and looking
  after it afterwards.

These describe how the thing works underneath. What it does *for a pilot* is the in-app
documentation at `/documentation`, written in [`frontend/src/docs/`](../frontend/src/docs/)
— that is the one to reach for when the question is "how do I use the map".

## Code layout

- `src/api/` — the HTTP boundary, one module per area of the API (`maps`, `systems`,
  `connections`, `signatures`, `watchlist`, `search`, `access`, `history`, `identity`,
  `reference`, …). Each owns its handlers, the routes that reach them, and the wire types
  it serves; `extract.rs` holds the request plumbing they share, and `router()` is the
  merge of their routers. Adding an endpoint means touching one file.
- `src/maps/` — the actions themselves: the authorized, validated, undoable commands the
  API calls into. This is where the rules live; the API layer only decodes and dispatches.
  `src/maps/store/` holds the SQL that writes placements, connections and signatures, one
  file per table: every full-row insert and every write more than one command makes, so a
  new column is changed there and nowhere else. It does SQL only; the commands keep the
  checks, events and undo.
- `src/esi/`, `src/sde/`, `src/seed/` — talking to EVE: the live API, the static data
  export, and loading the latter into the database.
- `src/alerts/`, `src/discord/` — what a map watches for, and where the notices go.
- `wsctl/` — the setup and management tool, a separate crate so the prompt toolkit it
  needs never reaches the server image. `install.sh` fetches its released binary.

## Working on it

Queries are checked at compile time against a live database, and the production image
builds from the `.sqlx` cache instead (`SQLX_OFFLINE=true`). Those two disagree the moment
a new query is added and the cache is not regenerated: everything passes locally and the
Docker build fails minutes into a deploy.

```sh
cargo sqlx prepare -- --all-targets   # after adding or changing a query
git config core.hooksPath .githooks   # once per clone: checks it before every push
```

The hook is in [`.githooks/pre-push`](../.githooks/pre-push). CI checks the same thing, in
the "builds without a database" job.
