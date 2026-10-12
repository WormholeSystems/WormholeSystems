# WormholeSystems

A **map** is a shared, real-time picture of a wormhole chain. Systems appear as nodes, the
wormholes between them appear as connections, and everyone with access sees the same
picture as it is scanned. When a corpmate pastes their probe scanner, resolves a hole, or
marks a connection end-of-life, it is on your screen before they have finished typing.

A Rust and SvelteKit rewrite of [wormhole.systems](https://wormhole.systems/).

---

> ## ⚠️ Pre-alpha
>
> **This is not ready to rely on.**
>
> - **Breaking changes are constant.** The schema, the API and the interface all change
>   without notice or a migration path. A deployment that worked yesterday may need to be
>   rebuilt from scratch today.
> - **There is no guarantee of continued updates.** Nothing here is promised, supported,
>   or committed to a release schedule.
> - **Your data is not safe.** Databases are wiped when it is convenient. Do not put a
>   chain you care about on it.
>
> Run it to look at it, or to help build it. Do not run it for a corp that needs it to
> work.

---

## What it does

- **Map the chain** — add systems and connections by hand, or let jump tracking log them
  as you fly.
- **Track signatures** — paste the probe scanner and the map syncs, then link each
  wormhole to the connection it turned out to be.
- **Watch each hole** — mass and lifetime per connection, so a fleet knows what a hole
  will still take.
- **Plan routes** — the shortest or safest way through the chain, with in-game waypoints
  set for you.
- **See your group** — where every tracked character is, live from ESI.
- **Control access** — per character, corporation or alliance, down to who may edit.
- **Get told** — Discord alerts for kills, proximity, and capitals in jump range.

## Running it

You need Docker, a domain pointed at the machine, and an EVE application from the
[developer portal](https://developers.eveonline.com/) with the `esi-location.*` scopes.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://install-next.wormhole.systems | sh
wsctl setup
```

`wsctl setup` checks the machine, asks for what it needs, writes `.env`, and brings the
stack up. It runs everything on its own (Postgres and Caddy included), or only the app
behind a Postgres and a reverse proxy you already have; `wsctl integrate` then prints what
that proxy needs. `wsctl update` takes new code and newer static data; `wsctl status` says what is
running and whether it answers. [docs/deployment.md](./docs/deployment.md) covers the rest.

## Developing

The whole stack, the same way it runs in production:

```sh
cp .env.example .env       # then fill in the EVE application and contact details
docker compose up -d --build
```

That serves http://localhost. What differs from production is only `.env`; see the
comments in `.env.example`. Every change needs another `--build`, so for quick iteration
run Postgres in Docker and the API and frontend on the host instead:

```sh
docker compose up -d db
cargo sqlx migrate run
cargo run -- seed          # the SDE: a few hundred MB the first time
cargo run                  # API on :3000
cd frontend && npm install && npm run dev
```

`cargo test` builds its own database per test and needs `DATABASE_URL` set. After changing
a query, `cargo sqlx prepare --workspace -- --all-targets` refreshes the offline cache that
CI compiles against; a pre-push hook checks it is current.

## Layout

- `src/` — the API. `src/maps/` holds the rules, `src/api/` only decodes and dispatches.
- `frontend/` — SvelteKit: the map canvas, the panels, and their state.
- `frontend/src/docs/` — the user documentation served at `/documentation`.
- `migrations/` — the schema, each file creating its tables in their final shape.
- `wsctl/` — the operator CLI that installs, updates and inspects a deployment.
- `docs/` — how it works underneath: data model, ESI endpoints, background processes.

## Contributing

The documentation is the easiest place to start: it is Markdown in `frontend/src/docs/`,
and a new page needs nothing but a file. See
[frontend/src/docs/README.md](./frontend/src/docs/README.md).

## Licence

MIT. See [LICENSE](./LICENSE).
