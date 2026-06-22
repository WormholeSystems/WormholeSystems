---
title: Self-hosting
category: Contributing & self-hosting
---

# Self-hosting

WormholeSystems is open source, and you can run your **own private instance** instead of (or alongside) using the public site at [wormhole.systems](https://wormhole.systems).

## The container repo

There's a ready-made Docker setup for it:

**[WormholeSystems/wormholesystems-containers](https://github.com/WormholeSystems/wormholesystems-containers)** — a Docker container setup to host a private wormhole.systems instance.

It bundles the app and the services it needs so you can stand up an instance with Docker rather than wiring everything together by hand. Follow the repo's README for the exact, up-to-date steps; at a high level you'll:

1. Clone the repository.
2. Register your **own EVE third-party application** for ESI so logins and tracking work against your instance.
3. Fill in the environment configuration (your EVE app credentials, app URL, and so on).
4. Bring the stack up with Docker.

## Restricting who can log in

By default, **anyone** with an EVE Online account can log into your instance. If you're running a private instance for your corp or alliance, you'll usually want to lock that down.

Set `ALLOWED_AFFILIATION_IDS` in the application environment to a comma-separated list of EVE IDs — any mix of **character**, **corporation**, and **alliance** IDs:

```bash
# Only pilots from these corporations/alliances (or this one character) may log in
ALLOWED_AFFILIATION_IDS=99000001,98000002,90000003
```

When a pilot signs in with EVE SSO, the app checks their character, its corporation, and its alliance against the list. If **any** of the three is present, they're allowed in; otherwise login is rejected with an "Access denied" message. The check applies to **every** login, including adding an alt character to an existing account.

Leave `ALLOWED_AFFILIATION_IDS` **empty** (the default) to allow anyone to log in.

> You can look up the numeric ID of a corporation or alliance from its [zKillboard](https://zkillboard.com) URL, or via [EVE Who](https://evewho.com).

Because the app caches its configuration, changes to this value take effect the next time the app container starts (it re-caches config on boot) — restart the stack after editing your environment. To apply a change without a full restart, re-cache the config in the running app container:

```bash
docker compose exec app php artisan optimize
```

## Why self-host

- **Your data, your server.** Everything stays on infrastructure you control — appealing for groups that want their intel kept in-house.
- **Independence.** Your chain doesn't depend on the public instance being up, and you can run whatever version you like.
- **Customisation.** Because you have the [source](/documentation/contributing-and-self-hosting/contributing), you can tweak and extend it for your group.

## Or just use the public site

The hosted instance at [wormhole.systems](https://wormhole.systems) is free, always up to date, and needs zero setup — it's the right choice for most groups. Self-hosting is there for those who specifically want their own.

Stuck on setup? [Ask in the Discord](/documentation/contributing-and-self-hosting/getting-help).
