# Deploying WormholeSystems

One machine, Docker, and a domain. `wsctl setup` asks two things about how the install is
put together, independently:

- The database: a Postgres container here (bundled), or one you already run.
- What faces the internet:
  - Caddy here, on ports 80 and 443, getting its own certificate. The default.
  - Caddy here behind your own proxy, as a plain-http path router: your proxy sends it the
    whole site over a Docker network or a loopback port.
  - Your proxy in Docker, reaching the API and the frontend on a network it shares with
    them.
  - Your proxy on this machine (nginx, a host Caddy), reaching them on loopback ports.

Everything below assumes both bundled unless it says otherwise. The other choices are in
[Choosing the pieces](#choosing-the-pieces).

## What runs

Four containers, from `docker-compose.yml`:

| container | what it is | reachable from |
|---|---|---|
| `caddy` | TLS and the single public origin | the internet, ports 80 and 443 |
| `web` | the SvelteKit server (adapter-node) | Caddy only |
| `api` | the Axum API and every background job | Caddy only |
| `db` | Postgres 16 | the other containers, and `127.0.0.1` on the host |

With scheduled backups on there is a fifth, `backup`, which dumps the database once a day
and reaches nothing else; see [Backups](#backups).

Caddy is the only thing published. `/api`, `/auth`, `/ws` and `/discord` go to the API;
everything else goes to the SvelteKit server. The database port is bound to the loopback so
`psql` works over SSH and nothing else can reach it.

## Before you start

A server with Docker Engine and Compose v2, 5GB of disk, and a DNS A record pointing at it.

## Images

The API and the frontend are published to GitHub's registry for amd64 and arm64, from
every push to main (`ghcr.io/wormholesystems/wormholesystems-api` and `-web`, tagged
`main` and `sha-<commit>`) and from every release tag (`1.2.3`, `1.2`, `latest`).
`wsctl setup` asks whether to pull those or build them here from the checkout, and writes
`WS_IMAGES=pull` or `build` to `.env`. A new install pulls by default.

Building wants about 2GB of memory for the Rust compile: on a 2GB machine, add swap first
or the compiler is killed part way through. Pulling needs none of that.

`WS_VERSION` picks the tag (`main` unless set) and `WS_IMAGE_PREFIX` the registry, for a
fork or a mirror. With `main`, the image can trail the checkout by however long CI takes
to build it after a push; pin a release to keep the two in step.

An install from before wsctl 0.6 has no `WS_IMAGES` and keeps building, as it always has.
To switch, set `WS_IMAGES=pull` in `.env` (or run `wsctl setup` again and pick it); the
next `wsctl update` pulls instead of compiling. `wsctl update` says so each time until the
choice is made.

## First run

Install `wsctl`, the setup tool:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://install-next.wormhole.systems | sh
```

It works out this machine's platform (Linux on x86_64 or arm64, macOS on Apple silicon or
Intel), checks the download against the `SHA256SUMS` published with the release, and drops
the binary in `/usr/local/bin`. Run inside a checkout, it offers to start the setup there;
anywhere else, it offers to clone one first. Pin a version with
`WSCTL_VERSION=wsctl-v0.6.0`; releases before 0.6 have no checksums and need
`WSCTL_SKIP_VERIFY=1`.

Then, in a checkout:

```sh
git clone https://github.com/WormholeSystems/wormholesystems-next.git
cd wormholesystems-next
wsctl setup
```

### Without questions

For a provisioning script or CI, `wsctl setup --non-interactive` asks nothing. Every answer
comes from the environment, then from an existing `.env`, then from the default the
question would have offered, and it is written to `.env` as the questions would have:

```sh
WS_CONTACT_NAME="Your Character" WS_CONTACT_EMAIL=you@example.com \
EVE_CLIENT_ID=... EVE_CLIENT_SECRET=... WS_DOMAIN=map.example.com \
  wsctl setup --non-interactive
```

The variables are the `.env` keys themselves (see [What is in `.env`](#what-is-in-env)),
and any other key `.env` can hold that starts with `WS_`, `EVE_`, `DISCORD_`,
`ANNOUNCEMENT_`, `ZKB_` or `KILLMAIL_` (or is one of the app's settings in
`.env.example`) is written there too. Required, and named together in one error when
missing:

- always: `WS_CONTACT_NAME`, `WS_CONTACT_EMAIL`, `EVE_CLIENT_ID`, `EVE_CLIENT_SECRET`
- `WS_DATABASE=external`: `WS_DATABASE_URL`
- `WS_PROXY=router`, `network` or `ports`: `WS_PUBLIC_URL`
- `DISCORD_CLIENT_ID` set: `DISCORD_APPLICATION_ID`, `DISCORD_PUBLIC_KEY`,
  `DISCORD_CLIENT_SECRET`

The choices take their usual defaults: everything bundled, Caddy with Let's Encrypt when
`WS_DOMAIN` is set, pulled images, scheduled backups on (`COMPOSE_PROFILES` without
`backup` turns them off), the killmail ingest only with `ZKB_LISTEN=1`, the announcement
only with `ANNOUNCEMENT_MESSAGE`. A plain `.env` written by hand works the same way: run
it with nothing in the environment and it takes `.env` as the answers.

### Building it instead

No release for your platform, no token, or you would rather not download a binary. The
checkout has the source, and Docker is already here:

```sh
docker run --rm -u "$(id -u):$(id -g)" -e CARGO_HOME=/tmp/cargo \
  -v "$PWD":/w -w /w rust:1.94-slim-bookworm cargo build --release -p wsctl
sudo install -m755 target/release/wsctl /usr/local/bin/wsctl
```

Running as yourself rather than root keeps `target/` yours; without `-u` the build leaves
root-owned files behind. With Rust already installed, `cargo run -p wsctl -- setup` does
the same thing without the container.

It checks the machine before asking for anything: Docker present and running, Compose 2.24
or later (the overlays in `deploy/` need it), disk space, and whether this checkout is
behind origin. Building the images here also wants about 2GB of memory and swap, and a
bundled database its loopback port (`DB_PORT`, 5432) free. `wsctl doctor` runs the same
checks at any time and changes nothing. Then it asks which database and
which proxy, the domain and the EVE SSO credentials, checks that the domain resolves to
this machine and that ports 80 and 443 are free, generates a database password, writes
`.env` with `chmod 600`, and brings the stack up.

Without a domain, Caddy serves plain http, and the callbacks need the address people open
it at. `setup` asks for it, offering `http://localhost` on the machine itself and the
server's own address when you are logged in over SSH.

The EVE application's callback URL has to match exactly what the CLI prints:
`https://your-domain/auth/callback`. Register it at
<https://developers.eveonline.com/applications>.

The first boot downloads CCP's static data export (~550MB) and seeds from it, which takes
a few minutes. It lands in a volume, so later restarts and rebuilds reuse it. Watch it with:

```sh
docker compose logs -f api
```

## Updating

```sh
wsctl update
```

Fast-forwards the checkout, pulls or rebuilds the images while the old containers keep
serving, dumps the database (see [Backups](#backups)), restarts, waits for the new
containers to report healthy, and takes CCP's static data when a newer build is out.
Migrations run on boot, so there is no separate step for those either.

It also brings `.env` up to date with the version it is running: an install set up by wsctl
0.5 is moved to the choices described in [From wsctl 0.5](#from-wsctl-05), and
`COMPOSE_FILE` is written again each time, so a `compose.override.yml` added since is
picked up.

An install from before wsctl 0.5 ran under its directory's name with a database role called
`vector`. Update wsctl first (the install one-liner again), then `wsctl update` moves it:
it stops the old containers, copies each old volume into a `wormholesystems_*` one and
renames the role and database inside the copy. The old volumes stay as they were, and it
prints the command to remove them once the new stack looks right. An older wsctl leaves the
API unable to log in to the database until the new one has run, and changes nothing else.

It also prunes the build cache and images the update left behind, keeping only what the
last day's builds used. Every api build leaves a full set of layers, and without this a
small disk fills up within a few weeks of updates.

The static data is the slow half: another ~550MB and a re-seed of a few minutes. It is
skipped when the loaded build is already current, and `--sde` fetches it regardless, which
is what to reach for if a download was interrupted. The fetch clears the old files first,
so it checks that CCP answers and that Docker has 2GB free before it starts; if the
download fails anyway, the API keeps running on the build its database has, and downloads
again the next time it starts.

```sh
wsctl update --sde
```

## Checking on it

```sh
wsctl status
```

Containers, how far behind origin the checkout is, which SDE build is loaded and whether
CCP has a newer one, and whether the public URL answers.

The API and the frontend have healthchecks, so `docker compose ps` says `healthy` rather
than only `running`. The API's is a request to `/healthz`, which answers 200 while the
server is up and its database answers and 503 otherwise; it is defined once, on the `api`
service in `docker-compose.yml`. `setup` and `update` start the stack with
`docker compose up -d --wait` and fail when something does not come up healthy, instead of
reporting a stack that started and then fell over. The first boot is allowed 20 minutes,
since the API seeds the static data before it answers.

Every container logs through Docker's `json-file` driver, capped at five files of 10MB
each. `WS_LOG_MAX_SIZE` and `WS_LOG_MAX_FILES` in `.env` change that. The API logs at
`info` by default; `RUST_LOG` in `.env` changes the level, for everything (`warn`) or per
module (`info,wormholesystems::killmails=debug`).

## Choosing the pieces

`docker-compose.yml` is the whole stack with everything bundled, and what compose reads
when nothing says otherwise. Every other choice is an overlay in `deploy/` on top of it,
and `wsctl setup` writes the ones an install needs into `COMPOSE_FILE` in `.env`, so plain
`docker compose` commands, `wsctl update` and `wsctl status` all run the same stack:

| file | when | what it does |
|---|---|---|
| `deploy/external-db.yml` | `WS_DATABASE=external` | no `db` container; the API connects to `WS_DATABASE_URL` |
| `deploy/db-network.yml` | the database is a container on another network | the API joins `WS_DATABASE_NETWORK` |
| `deploy/router-network.yml` | `WS_PROXY=router`, proxy in Docker | Caddy publishes nothing and joins `WS_PROXY_NETWORK` as `wormholesystems-router:80` |
| `deploy/router-port.yml` | `WS_PROXY=router`, proxy on the host | Caddy is published on `127.0.0.1:WS_ROUTER_PORT` (8080) |
| `deploy/app-network.yml` | `WS_PROXY=network` | no Caddy; the API and the frontend join `WS_PROXY_NETWORK` as `wormholesystems-api:3000` and `wormholesystems-web:3001` |
| `deploy/app-ports.yml` | `WS_PROXY=ports`, or alongside the network | no Caddy; the two are published on `127.0.0.1:3000` and `:3001` |

An overlay that drops `db` or `caddy` does it with a profile nothing turns on, so a
container left from before keeps running until you remove it: `wsctl setup` says so and
prints `docker compose rm -sf db caddy`. Files you add to `COMPOSE_FILE` yourself are kept
after wsctl's own.

The `sde` and `uploads` volumes are the same whichever pieces are picked, so switching
keeps the static data and the uploads. The database is the one thing that has to be moved;
see [Moving the database out](#moving-the-database-out).

### Your own Postgres

`wsctl setup` asks for the URL, as the containers reach it: the database's name on a
shared Docker network, or the machine's address. Never `localhost`, which inside a
container is the container. When the database is a container on a network of its own, name
that network too and the API joins it. `setup` and `doctor` check that it answers from
where the API will look.

The database has to exist, with a role that owns it; the API creates its tables on boot.
On a shared Postgres, for example:

```sql
CREATE ROLE wormholesystems LOGIN PASSWORD '...';
CREATE DATABASE wormholesystems OWNER wormholesystems;
REVOKE CONNECT ON DATABASE wormholesystems FROM PUBLIC;
```

### Behind your own proxy

All of them need the public URL, the address your proxy serves, scheme included. The EVE
and Discord callbacks are built from it and must match it exactly. `wsctl setup` creates
the Docker network when it is missing.

With Caddy as the router, your proxy sends everything to one upstream and Caddy splits the
paths, so nothing about the app's routes has to live in your proxy's config. Caddy believes
the `X-Forwarded-For` and `-Proto` your proxy sets when it connects from a private address
(`WS_TRUSTED_PROXIES` changes that), so the API sees the visitor's address.

Without it, `/api`, `/auth`, `/ws` and `/discord` go to the API with WebSocket upgrades
passed through, everything else to the frontend.

```sh
wsctl integrate
```

prints what your proxy needs for the choice this install made: a Caddy site block, a
Traefik (or Coolify) label set to put in `compose.override.yml`, and an nginx server block,
whichever apply.

### From wsctl 0.5

An install set up by wsctl 0.5 has `WS_SETUP=full` or `WS_SETUP=standalone` in `.env`.
Full is both bundled, and standalone an external database behind your proxy on the
network (plus the loopback ports when its `COMPOSE_FILE` had
`compose.standalone.ports.yml`). The first `wsctl update` from 0.6 rewrites `.env` to
`WS_DATABASE` and `WS_PROXY` and points `COMPOSE_FILE` at the overlays, which run the same
containers under the same names on the same networks. `compose.standalone.yml` stays in
the checkout, so an older wsctl, or a `.env` that still names it, keeps working.

### Moving the database out

The data moves with a dump. With the API stopped, so nothing is written after it:

```sh
docker compose stop api
wsctl backup                         # prints the dump's name
wsctl setup                          # pick your own Postgres; starts the app on it, empty
wsctl restore backups/wormholesystems-...-manual.dump
docker compose rm -sf db
```

The new database has to exist and be reachable, as above. The map is down from the `stop`
until the restore has started the API again, and between `setup` and the restore it runs
on the empty database. The `pgdata` volume stays until you remove it yourself.

## Changes of your own

Anything you change in a tracked file is in the way of the next pull, so `wsctl update`
stops on it. Two places are yours instead, and survive every update:

- `compose.override.yml` next to `docker-compose.yml`, for compose changes: labels for
  your Traefik, another network, resource limits, an environment variable. Compose reads it
  on its own while `COMPOSE_FILE` is unset; once it is set, wsctl adds the file to the end
  of it, so it wins. Added after setup, it is picked up by the next `wsctl update` (or name
  it in `COMPOSE_FILE` yourself).
- `caddy.d/` for the bundled Caddy: `*.site` files are imported into the WormholeSystems
  site block (a header, a redirect, a `handle` of your own), `*.caddy` files at the top
  level (another site on the same Caddy). `docker compose restart caddy` reloads them.

For example, `caddy.d/hsts.site`, once https is what you will always serve:

```
header Strict-Transport-Security "max-age=31536000"
```

Neither is tracked.

## Certificates

Caddy gets them from Let's Encrypt on first start and renews them on its own. Two things
have to hold or issuance fails: the domain resolves to this machine, and port 80 reaches
Caddy from the internet. The certificates live in the `caddy_data` volume; keep it. Losing
it means re-issuing, and Let's Encrypt rate-limits that.

Behind Cloudflare's proxy, DNS resolves to Cloudflare rather than the server, so `setup`
warns rather than refuses. Either turn the proxy off while the certificate is issued, or use
Cloudflare's origin certificate as your own (below).

Let's Encrypt is the default, not the only choice. `WS_TLS` says where the bundled Caddy's
certificate comes from, and `wsctl setup` asks when there is a domain:

| `WS_TLS` | what |
|---|---|
| `acme` | Let's Encrypt, as above. The default. |
| `files` | Your own: `certs/fullchain.pem` (the certificate with its chain) and `certs/privkey.pem` in the checkout, PEM encoded. `WS_TLS_CERT` and `WS_TLS_KEY` name other files in `certs/`. Nothing in `certs/` is committed. |
| `internal` | Caddy's own CA. Browsers warn until they are told to trust it; for a private network. |
| `off` | Plain http on `HTTP_PORT`, for something in front that does TLS. |

With anything but `acme` the ports can be other than 80 and 443, and the name only has to
resolve wherever the browsers are. Change it in `.env` and run `docker compose up -d
caddy`; a certificate file replaced in place needs `docker compose restart caddy`.

## What is in `.env`

`setup` writes these; the rest of the file is the same as `.env.example`.

| key | why |
|---|---|
| `WS_DOMAIN` | Caddy's site address. Blank serves plain http on `HTTP_PORT`. |
| `HTTP_PORT`, `HTTPS_PORT` | What Caddy publishes. 80 and 443 in production. |
| `WS_TLS` | Where Caddy's certificate comes from: `acme`, `files`, `internal` or `off`. See [Certificates](#certificates). |
| `WS_CONTACT_NAME`, `WS_CONTACT_EMAIL` | Who runs this install. Every request to ESI, zKillboard and EVE Ref carries them, which is how those services tell operators apart and reach you. The server refuses to start without them. |
| `EVE_CLIENT_ID`, `EVE_CLIENT_SECRET` | The SSO application. |
| `EVE_REDIRECT_URI` | Derived from the domain; must match the application exactly. |
| `POSTGRES_PASSWORD` | Bundled database only. Generated once. Changing it means changing it in Postgres too. |
| `WS_IMAGES` | `pull` the published images or `build` them here. Missing means build. |
| `WS_VERSION`, `WS_IMAGE_PREFIX` | Which published images: the tag (`main`) and where they live. |
| `WS_DATABASE` | `bundled` or `external`. |
| `WS_DATABASE_URL` | External only: the Postgres to use, as the containers reach it. |
| `WS_DATABASE_NETWORK` | External only, optional: the Docker network the database is on. |
| `WS_PROXY` | `caddy`, `router`, `network` or `ports`. |
| `WS_PUBLIC_URL` | The address people open, when it is not `https://WS_DOMAIN`; the callbacks derive from it. |
| `WS_PROXY_NETWORK` | `router` and `network`: the Docker network your proxy finds the app on. |
| `WS_ROUTER_PORT` | `router` on the host: Caddy's loopback port. |
| `WS_API_PORT`, `WS_WEB_PORT` | `ports`: the API's and the frontend's loopback ports. |
| `WS_TRUSTED_PROXIES` | `router` only, optional: whose forwarded headers Caddy believes. `private_ranges` unless set. |
| `COMPOSE_FILE` | The compose files to run, when anything is not bundled. wsctl keeps it in step with the above. |
| `COMPOSE_PROFILES` | `backup` turns the scheduled dumps on. |
| `WS_BACKUP_DIR`, `WS_BACKUP_EVERY_HOURS`, `WS_BACKUP_KEEP_DAYS` | Where dumps go (`./backups`), how often the scheduled ones run (24) and how long they are kept (14 days). |
| `WS_BACKUP_PG_VERSION` | The Postgres client the dumps use (16). wsctl raises it for a newer database of your own. |

`ALLOWED_AFFILIATION_IDS` and `MAP_CREATOR_AFFILIATION_IDS` are asked for too; see
[Who may use it](#who-may-use-it).

Discord, the killmail ingest and the [announcement banner](#announcements) are optional and
off unless configured; `setup` asks about each. The killmail ingest (`ZKB_LISTEN=1`) follows
zKillboard's feed and fills in `KILLMAIL_BACKFILL_DAYS` (90) of history on its first start;
kill alerts, kills on the map and the threat view all read from it and stay empty without
it. `.env.example` lists the rest: `ZKB_BASE_URL`, `EVE_SCOUT_URL`,
`DATABASE_MAX_CONNECTIONS`, `SERVER_STATUS_POLL_SECS`, `SKYHOOK_POLL_SECS`, none of which a
normal install needs.
`UPLOADS_DIR` (default `data/uploads`) is where uploaded background images land. In the
container that path is the `uploads` volume, so leave it alone unless the API runs outside
Docker.

## Who may use it

Anyone with an EVE account can log in to a fresh install. To keep it to your own people, set
`ALLOWED_AFFILIATION_IDS` to a comma-separated list of character, corporation and alliance
ids:

```sh
ALLOWED_AFFILIATION_IDS=99000001,98000002,2112000001
```

A character gets in when its own id, its corporation's or its alliance's is on the list,
checked against ESI at every login. Anyone else is sent back to the login page with an
explanation, and nothing about them is stored. The same goes for adding an alt to an
account: a character outside the list is not added. The ids are on the corporation's or
alliance's page at zKillboard or EVE Who. Empty, the default, lets everyone in.

It is read when the API starts, so restart it after a change (`docker compose up -d api`).
An entry that is not a number stops the API from starting rather than being skipped, since
a list that is only a typo would otherwise read as empty and let everyone in.

The check is at login only. Somebody who is already signed in keeps their session until it
runs out (30 days) or they log out, and characters already on an account stay on it. To
cut somebody off at once, delete their sessions:

```sh
docker compose exec db psql -U wormholesystems wormholesystems -c \
  "delete from sessions where user_id = (select user_id from characters where name = 'Their Name')"
```

Map creation can be limited the same way, independently: an instance can let a whole
alliance in while only its leadership creates maps.

```sh
MAP_CREATOR_AFFILIATION_IDS=98000002
```

It is matched against the character a map is created as, the active one, not every
character on the account: a director's alt in an NPC corporation does not get to create
maps. It covers importing a file as a new map too. Off the list, the maps page simply has
no create or import buttons, and the API answers 403. Maps that already exist are not
affected, and anyone given a role on one works on it as before. Empty, the default, lets
everyone who can log in create maps.

## Announcements

A banner above every page, the landing page included, for planned downtime, a move to a new
address, or anything else people should read before they use the map. `setup` asks for it;
it lives in `.env`, so changing it later is an edit there and a restart of the API
(`docker compose up -d api`).

| key | what |
|---|---|
| `ANNOUNCEMENT_MESSAGE` | The text, in quotes. Empty turns the banner off. |
| `ANNOUNCEMENT_TITLE` | A bold line above it, optional. |
| `ANNOUNCEMENT_LEVEL` | `info` (the default), `warning` or `critical`; the colour. Anything else reads as `info`. |
| `ANNOUNCEMENT_LINK_URL`, `ANNOUNCEMENT_LINK_LABEL` | A link under the text. The label defaults to the url; a label without a url shows nothing. |
| `ANNOUNCEMENT_DISMISSIBLE` | `true` (the default) lets people close it; `false` keeps it up. |

Closing it is remembered in a cookie against that exact announcement, so changing any of its
text, its level or its link brings it back for everyone. The cookie is read while the page
is rendered on the server, so a closed banner is never drawn and then taken away again.

## Discord

Optional, and `setup` offers it. WormholeSystems uses a Discord application for three things: linking
an account so `/wh` knows who you are, the slash commands themselves, and posting alerts
to a channel. Alerts to a webhook work without any of this.

At <https://discord.com/developers/applications>, create an application and take:

| where | what | into |
|---|---|---|
| General Information | Application ID | `DISCORD_APPLICATION_ID` |
| General Information | Public Key | `DISCORD_PUBLIC_KEY` |
| OAuth2 | Client ID and Client Secret | `DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET` |
| Bot | Token, only to post as the bot or send DMs | `DISCORD_BOT_TOKEN` |

Add `https://your-domain/discord/callback` as an OAuth2 redirect, exactly.

Two things have to wait until the stack is running, because Discord checks them:

1. Set the Interactions Endpoint URL to `https://your-domain/discord/interactions`. Discord
   signs a ping at it and refuses to save if it does not answer.
2. `wsctl discord-register` uploads the `/wh` command: account, alerts, route, and the
   `alert-dm` and `alert-channel` groups that create alerts from a channel. It is
   registered globally, so Discord takes a few minutes to show it.

## Backups

The database is what matters, and wsctl dumps it three ways, all with `pg_dump` in a
container of the `backup` service, against whichever database the API uses (bundled or
your own), into `backups/` in the checkout (`WS_BACKUP_DIR` moves it):

- `wsctl update` takes one before it starts the new version, since migrations run as the
  API boots and none of them has a way back. If the dump fails, nothing is restarted;
  `wsctl update --no-backup` goes ahead without it.
- `wsctl backup` takes one now.
- With scheduled backups on (`setup` asks; it is the `backup` profile in
  `COMPOSE_PROFILES`), the `backup` container takes one every 24 hours
  (`WS_BACKUP_EVERY_HOURS`) and deletes those older than 14 days (`WS_BACKUP_KEEP_DAYS`).

Each is a `pg_dump` custom-format file, `wormholesystems-<UTC time>-<why>.dump`, readable
only by whoever owns the backup directory. Copy them somewhere that is not this machine.

```sh
wsctl restore backups/wormholesystems-20261014-031500-scheduled.dump
```

stops the API, empties the database and restores the dump into it, then starts the API
again, which migrates it forward if the dump is from an older version. It asks first;
`--yes` does not. A dump from elsewhere is mounted in read-only, wherever it is.

`pg_dump` refuses a server newer than itself. The bundled database is Postgres 16, and so
is the client by default; for your own, wsctl asks its version before the first dump and
sets `WS_BACKUP_PG_VERSION` to match when it is newer.

The one thing outside the database is the `uploads` volume: the background images people
put behind their maps. Losing it costs each of them a picture and nothing else; a settings
row that points at a missing file simply shows the plain grid again. Copy it out with
`docker compose cp api:/app/data/uploads ./uploads` if it is worth keeping.
