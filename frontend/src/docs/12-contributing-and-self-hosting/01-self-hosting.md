---
title: Self-hosting
category: Contributing & self-hosting
---

# Self-hosting

WormholeSystems is open source, and you can run your own private instance. Everything a
deployment needs is in the one repository, and an operator CLI called **wsctl** does the
work.

> **This is pre-alpha software.** Breaking changes land constantly, there is no promise of
> continued updates, and databases get wiped when it is convenient. Host it to look at it,
> not because a corp is depending on it.

## What you need

- A machine with **Docker**, and a **domain** pointed at it. Certificates are obtained for
  you, so the domain has to resolve before you start.
- Your own **EVE application** from the
  [developer portal](https://developers.eveonline.com/). Logins and tracking run against
  your instance, not the public one, so it needs its own credentials and its own callback
  URL.

## Setting it up

```sh
curl --proto '=https' --tlsv1.2 -sSf https://install-next.wormhole.systems | sh
wsctl setup
```

`wsctl setup` checks the machine first — Docker, disk, ports, DNS — then asks for the
domain, your EVE credentials, and optionally Discord. It writes them to `.env`, generates a
database password, and brings the stack up. The first boot downloads CCP's static data
export, which is a few hundred megabytes and takes a while.

## Restricting who can log in

By default anyone with an EVE account can log in to your instance. For a corp or alliance
instance, set `ALLOWED_AFFILIATION_IDS` in `.env` to a comma-separated list of character,
corporation and alliance ids (`wsctl setup` asks for it):

```sh
ALLOWED_AFFILIATION_IDS=99000001,98000002,2112000001
```

A pilot gets in when their character, its corporation or its alliance is on the list. That
holds for adding an alt to an account too. Leave it empty to let everyone in. Restart the
API after changing it; people already signed in stay signed in until their session ends.

## Restricting who can create maps

`MAP_CREATOR_AFFILIATION_IDS` takes the same kind of list and limits who may create (or
import) a map, separately from who may log in. It is checked against the character you are
currently acting as. Everyone else can still work on the maps they are given access to; they
just do not see the button to make one. Empty lets everyone create maps.

## Looking after it

| Command        | What it does                                                          |
| -------------- | --------------------------------------------------------------------- |
| `wsctl status` | What is running, how far behind the code is, whether the URL answers. |
| `wsctl update` | Takes new code and newer static data, rebuilds, restarts.             |
| `wsctl doctor` | Checks the machine and changes nothing.                               |

Database migrations run when the app boots, so there is no separate step for them.

## What the database holds

Linking a character stores the refresh token EVE hands back, unencrypted. Anyone who can
read the database can act as any linked character, within the scopes that character
granted. Treat the database as being as sensitive as the pilots on it: keep the port
closed to the outside, and do not hand a dump to somebody you would not hand your API keys
to.

Nothing backs itself up yet. If you are keeping a chain you care about — see the warning
above about not doing that — take your own dumps.
