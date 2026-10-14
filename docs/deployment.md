# Deploying WormholeSystems

One machine, Docker, and a domain. There are two setups, and `wsctl setup` asks which:

- Full: everything on this machine, Postgres and Caddy included. Caddy gets the
  certificate. This is what a fresh server wants, and what everything below assumes unless
  it says otherwise.
- Standalone: only the API and the frontend, for a machine that already runs a Postgres and
  a reverse proxy for other things. See [Standalone](#standalone).

## What runs

Four containers, from `docker-compose.yml`:

| container | what it is | reachable from |
|---|---|---|
| `caddy` | TLS and the single public origin | the internet, ports 80 and 443 |
| `web` | the SvelteKit server (adapter-node) | Caddy only |
| `api` | the Axum API and every background job | Caddy only |
| `db` | Postgres 16 | the other containers, and `127.0.0.1` on the host |

Caddy is the only thing published. `/api`, `/auth`, `/ws` and `/discord` go to the API;
everything else goes to the SvelteKit server. The database port is bound to the loopback so
`psql` works over SSH and nothing else can reach it.

## Before you start

A server with Docker Engine and Compose v2, 5GB of disk, and a DNS A record pointing at it.
The release build wants about 2GB of memory: on a 2GB machine, add swap first or the
compiler is killed part way through.

## First run

Install `wsctl`, the setup tool:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://install-next.wormhole.systems | sh
```

It works out this machine's platform, drops the binary in `/usr/local/bin`, and offers to
run the setup straight away. Pin a version with `WSCTL_VERSION=wsctl-v0.1.0`.

Then, in a checkout:

```sh
git clone https://github.com/WormholeSystems/wormholesystems-next.git
cd wormholesystems-next
wsctl setup
```

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

It checks the machine before asking for anything: Docker present and running, compose v2,
disk space, and whether this checkout is behind origin. Then it asks for the domain and the
EVE SSO credentials, checks that the domain resolves to this machine and that ports 80 and
443 are free, generates a database password, writes `.env` with `chmod 600`, and brings the
stack up.

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

Fast-forwards the checkout, rebuilds, restarts, and takes CCP's static data when a newer
build is out. Migrations run on boot, so there is no separate step for those either.

It also prunes the build cache and images the rebuild left behind, keeping only what the
last day's builds used. Every api build leaves a full set of layers, and without this a
small disk fills up within a few weeks of updates.

The static data is the slow half: another ~550MB and a re-seed of a few minutes. It is
skipped when the loaded build is already current, and `--sde` fetches it regardless, which
is what to reach for if a download was interrupted:

```sh
wsctl update --sde
```

## Checking on it

```sh
wsctl status
```

Containers, how far behind origin the checkout is, which SDE build is loaded and whether
CCP has a newer one, and whether the public URL answers.

## Standalone

For a machine that already has a Postgres and a reverse proxy, where a second database and
a second thing on ports 80 and 443 would be in the way. Two containers run, from
`compose.standalone.yml`:

| container | what it is | reachable from |
|---|---|---|
| `web` | the SvelteKit server | the Docker network you name, as `wormholesystems-web:3001` |
| `api` | the Axum API and every background job | the same network, as `wormholesystems-api:3000` |

`wsctl setup` asks for the public URL (whatever your proxy serves, scheme included), the
database URL, and the Docker network to join, which it creates if it is missing. It writes
`COMPOSE_FILE` to `.env`, so plain `docker compose` commands, `wsctl update` and
`wsctl status` all run the standalone stack from then on. A proxy that runs on the machine
itself rather than in Docker can also have the two published on `127.0.0.1`
(`compose.standalone.ports.yml`, ports 3000 and 3001 unless you pick others).

The database has to exist, with a role that owns it; the API creates its tables on boot.
On a shared Postgres, for example:

```sql
CREATE ROLE wormholesystems LOGIN PASSWORD '...';
CREATE DATABASE wormholesystems OWNER wormholesystems;
REVOKE CONNECT ON DATABASE wormholesystems FROM PUBLIC;
```

The URL is how the containers reach it: the database's name on the shared Docker network,
or the machine's address. Never `localhost`, which inside a container is the container.
`setup` and `doctor` check that it answers from the network.

Then point the proxy at it. `/api`, `/auth`, `/ws` and `/discord` go to the API with
WebSocket upgrades passed through, everything else to the frontend:

```sh
wsctl integrate
```

prints that, with a Caddy site block for a Caddy on the same network and an nginx server
block for the loopback ports. The EVE and Discord callbacks are built from the public URL
and must match it exactly, as in the full setup.

### From full to standalone

The data moves with a dump, into the new database while it is still empty: the API creates
its tables on boot, so restore before the switch, not after. From the checkout, still on
the full setup:

```sh
docker compose stop api
docker compose exec -T db pg_dump -U vector -Fc vector > vector.dump
pg_restore --no-owner --role=wormholesystems -d "postgres://wormholesystems:...@host/wormholesystems" vector.dump
wsctl setup                          # pick standalone; starts the app on the restored data
docker compose up -d --remove-orphans
```

The map is down from the `stop` until `setup` has the new stack up. `pg_restore` is any
client that reaches the new database, for example one in a container on the same network.
After the switch compose no longer counts the full stack's `db` and `caddy` as part of the
project, so they keep running until `--remove-orphans` takes them away; the `pgdata` volume
stays until you remove it yourself. The `sde` and `uploads` volumes are shared by both
setups and carry over as they are.

## Certificates

Caddy gets them from Let's Encrypt on first start and renews them on its own. Two things
have to hold or issuance fails: the domain resolves to this machine, and port 80 reaches
Caddy from the internet. The certificates live in the `caddy_data` volume; keep it. Losing
it means re-issuing, and Let's Encrypt rate-limits that.

Behind Cloudflare's proxy, DNS resolves to Cloudflare rather than the server, so `setup`
warns rather than refuses. Either turn the proxy off while the certificate is issued, or use
Cloudflare's own certificate and put Caddy behind it.

## What is in `.env`

`setup` writes these; the rest of the file is the same as `.env.example`.

| key | why |
|---|---|
| `WS_DOMAIN` | Caddy's site address. Blank serves plain http, which is only useful locally. |
| `HTTP_PORT`, `HTTPS_PORT` | What Caddy publishes. 80 and 443 in production. |
| `WS_CONTACT_NAME`, `WS_CONTACT_EMAIL` | Who runs this install. Every request to ESI, zKillboard and EVE Ref carries them, which is how those services tell operators apart and reach you. The server refuses to start without them. |
| `EVE_CLIENT_ID`, `EVE_CLIENT_SECRET` | The SSO application. |
| `EVE_REDIRECT_URI` | Derived from the domain; must match the application exactly. |
| `POSTGRES_PASSWORD` | Full only. Generated once. Changing it means changing it in Postgres too. |
| `WS_SETUP` | `full` or `standalone`. Missing means full. |
| `COMPOSE_FILE` | Standalone only: the compose files to run instead of `docker-compose.yml`. |
| `WS_PUBLIC_URL` | Standalone only: the address your proxy serves; the callbacks derive from it. |
| `WS_DATABASE_URL` | Standalone only: the Postgres to use, as the containers reach it. |
| `WS_PROXY_NETWORK` | Standalone only: the Docker network the proxy finds the app on. |
| `WS_API_PORT`, `WS_WEB_PORT` | Standalone only, when published on the loopback. |

`ALLOWED_AFFILIATION_IDS` and `MAP_CREATOR_AFFILIATION_IDS` are asked for too; see
[Who may use it](#who-may-use-it).

Discord, the killmail ingest and the [announcement banner](#announcements) are optional and
off unless configured; see `.env.example`.
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
docker compose exec db psql -U vector vector -c \
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
address, or anything else people should read before they use the map. `setup` does not ask
for it; set it in `.env` and restart the API (`docker compose up -d api`).

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

Not automated. The database is what matters. In the full setup:

```sh
docker compose exec -T db pg_dump -U vector vector | gzip > wormholesystems-$(date +%F).sql.gz
```

Restoring is the same in reverse, into a stopped stack with an empty database.

The one thing outside it is the `uploads` volume: the background images people put behind
their maps. Losing it costs each of them a picture and nothing else; a settings row that
points at a missing file simply shows the plain grid again. Copy it out with
`docker compose cp api:/app/data/uploads ./uploads` if it is worth keeping.
