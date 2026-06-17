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

## Why self-host

- **Your data, your server.** Everything stays on infrastructure you control — appealing for groups that want their intel kept in-house.
- **Independence.** Your chain doesn't depend on the public instance being up, and you can run whatever version you like.
- **Customisation.** Because you have the [source](/documentation/contributing-and-self-hosting/contributing), you can tweak and extend it for your group.

## Or just use the public site

The hosted instance at [wormhole.systems](https://wormhole.systems) is free, always up to date, and needs zero setup — it's the right choice for most groups. Self-hosting is there for those who specifically want their own.

Stuck on setup? [Ask in the Discord](/documentation/contributing-and-self-hosting/getting-help).
