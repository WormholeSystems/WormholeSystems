# WormholeSystems

[![tests](https://github.com/WormholeSystems/WormholeSystems/actions/workflows/tests.yml/badge.svg)](https://github.com/WormholeSystems/WormholeSystems/actions/workflows/tests.yml)
[![linter](https://github.com/WormholeSystems/WormholeSystems/actions/workflows/lint.yml/badge.svg)](https://github.com/WormholeSystems/WormholeSystems/actions/workflows/lint.yml)
[![License](https://img.shields.io/github/license/WormholeSystems/WormholeSystems)](LICENSE)
[![Stack](https://img.shields.io/badge/self--host-wormholesystems--containers-blue)](https://github.com/WormholeSystems/wormholesystems-containers)
[![wsctl](https://img.shields.io/github/v/release/WormholeSystems/wormholesystems-cli?label=wsctl)](https://github.com/WormholeSystems/wormholesystems-cli)

Wormhole mapping and tracking for EVE Online — live at [wormhole.systems](https://wormhole.systems). Real-time chain maps, signatures, character tracking and killmail intel, built with Laravel 12, Inertia.js, Vue 3 and Tailwind CSS.

## Self-hosting

To run your own instance you don't need this repository directly — use the [container stack](https://github.com/WormholeSystems/wormholesystems-containers) with its interactive setup wizard:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://install.wormhole.systems | sh
```

The rest of this README is about developing the application itself.

## Development setup

**Requirements:** PHP 8.4+, Composer, MySQL/MariaDB, Redis, Node.js + npm. We strongly recommend [Laravel Herd](https://herd.laravel.com/), which provides all of it pre-configured with automatic HTTPS.

```bash
git clone https://github.com/WormholeSystems/WormholeSystems.git
cd WormholeSystems

cp .env.example .env
composer install
npm install
php artisan key:generate
php artisan migrate

# EVE static data (~500MB download; seeding may need more memory)
php artisan sde:download
php artisan sde:prepare
php -d memory_limit=2G artisan db:seed
```

In Herd, make sure **MySQL** and **Redis** are running and create a database named `wormholesystems`. Herd serves the site at `https://wormholesystems.test` (derived from the folder name).

Key `.env` values for local development:

```env
APP_URL=https://wormholesystems.test
DB_DATABASE=wormholesystems
DB_USERNAME=root
DB_PASSWORD=

# CCP requires contact info on third-party apps — leaving this empty
# risks an EVE API ban. Format: "you@example.com | Your EVE Character"
CONTACT_EMAIL=

# From your EVE developer application (see below)
EVE_CLIENT_ID=
EVE_CLIENT_SECRET=

# Reverb via Herd's Broadcasting service
REVERB_APP_ID=1001
REVERB_APP_KEY=laravel-herd
REVERB_APP_SECRET=secret
REVERB_HOST="reverb.herd.test"
REVERB_PORT=443
REVERB_SCHEME=https
```

### Running it

```bash
npm run dev                 # frontend with hot reload
php artisan queue:work      # background jobs (killmails, locations, ...)
php artisan schedule:work   # scheduled tasks
```

Real-time features need **Reverb**: in Herd's Services panel, add "Reverb" under _Broadcasting_ and start it (not enabled by default; runs at `reverb.herd.test:443`).

## EVE application setup

SSO login requires an application at [developers.eveonline.com](https://developers.eveonline.com/):

- **Connection type:** Authentication & API Access
- **Callback URL:** `https://wormholesystems.test/eve/callback`
- **Scopes:**
    - `publicData`
    - `esi-location.read_location.v1` — character location
    - `esi-location.read_online.v1` — online status
    - `esi-location.read_ship_type.v1` — current ship
    - `esi-ui.write_waypoint.v1` — set autopilot waypoints

Copy the Client ID and Secret into `EVE_CLIENT_ID` / `EVE_CLIENT_SECRET`. Login works without the ESI scopes, but character tracking and autopilot features need them.

## Discord application setup

Account linking, slash commands and personal proximity alerts require an application at the [Discord Developer Portal](https://discord.com/developers/applications):

1. Create an application and open **OAuth2**. Add `https://wormholesystems.test/discord/callback` as a redirect URL. Use the production domain instead for a production deployment.
2. Open **Bot**, create or reset the bot token, and keep it secret.
3. Under **Installation**, enable Guild Install with the `applications.commands` and `bot` scopes. Grant the bot **View Channels**, **Send Messages**, and **Embed Links** permissions for channel alerts.
4. Install the application in the Discord servers where commands should be available.

Channel alerts can optionally use Discord's native role picker. The selected role must be marked **Mentionable** in the server for the bot to notify its members. Granting the bot **Mention Everyone** also permits non-mentionable role pings, but is not recommended.

Map managers configure alert rules, bot-managed alert oversight, delivery destinations, and role mentions on the map's **Discord settings** page at `/maps/{map}/settings/discord`. Delivery destinations may use Discord channel webhook URLs; these webhooks remain one delivery type within the broader Discord configuration.

Configure the application credentials in `.env`:

```dotenv
DISCORD_APPLICATION_ID=           # Application ID from General Information
DISCORD_CLIENT_ID=                # Usually the same value as the Application ID
DISCORD_CLIENT_SECRET=            # OAuth2 client secret
DISCORD_BOT_TOKEN=                # Token from the Bot page
DISCORD_CALLBACK="${APP_URL}/discord/callback"

# Optional: use guild-scoped commands for immediate updates during development
DISCORD_TEST_GUILD_ID=
```

Register the commands globally for production:

```bash
php artisan discord:register-commands --global
```

For development, set `DISCORD_TEST_GUILD_ID` and omit `--global`. Guild commands update immediately, while global command changes can take time to propagate.

The bot is a long-running CLI process and must run separately from queue workers, Reverb and Octane:

```bash
php artisan discord:listen
```

Production deployments must supervise exactly one `discord:listen` process. The process exits cleanly on `SIGTERM`/`SIGQUIT` and participates in `php artisan reload` through `discord:restart`.

## Background services

- **Queue** — killmail ingest (zKillboard), character location/online updates, sovereignty data. `php artisan queue:failed` / `queue:retry all` for failures.
- **Scheduler** — server status and character updates (seconds/minutes cadence), signature cleanup, sovereignty and killmail backfills. `php artisan schedule:list` shows everything.
- **Reverb** — WebSockets for real-time map updates. Without Herd: `php artisan reverb:start`.
- **Discord bot** — slash commands and personal alerts. Run one supervised `php artisan discord:listen` process.

## Useful commands

```bash
php artisan test                       # test suite
vendor/bin/pint                        # code formatting
php artisan optimize:clear             # clear all caches
php artisan tinker                     # interactive shell
php artisan migrate:fresh --seed       # reset database (destructive)
php artisan app:listen-for-killmails   # live killmail stream
php artisan app:get-killmails-for-day 2026-01-15
php artisan discord:register-commands --global
php artisan discord:listen
```

## Troubleshooting

- **SDE seeding runs out of memory** → `php -d memory_limit=2G artisan db:seed`
- **No real-time updates** → Reverb running? Check the WebSocket connection in the browser console.
- **Queue jobs not processing** → Redis running? Worker active (`php artisan queue:work`)?
- **EVE SSO fails** → Client ID/Secret correct, callback URL matches the developer application exactly, scopes granted. Logs: `storage/logs/laravel.log`.
- **Discord linking fails** → Client ID/Secret and callback URL match the Discord application exactly.
- **Slash commands are missing** → Install the application with `applications.commands`, then register commands globally or against `DISCORD_TEST_GUILD_ID`.
- **Discord alerts are not delivered** → Bot process and queue worker running? Check channel permissions and the configured bot token.

## Contributing

Fork, branch, make your changes, then `php artisan test` and `vendor/bin/pint` before opening a pull request — CI runs both.

## Related repositories

- [wormholesystems-containers](https://github.com/WormholeSystems/wormholesystems-containers) — production docker stack
- [wormholesystems-cli](https://github.com/WormholeSystems/wormholesystems-cli) — `wsctl` setup and management tool

## License

This project is open-sourced software licensed under the [MIT license](LICENSE).
