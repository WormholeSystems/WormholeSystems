---
title: Using the API
---

# Using the API

Anything the map does, a script can do too. The map screen talks to WormholeSystems over
a JSON API, and every part of it accepts a personal access token in place of being signed
in. There is no separate, smaller API for scripts: what the screen uses is what you use.

The API of the previous version of WormholeSystems is not carried over. A tool written
against it needs to be pointed at the endpoints described below.

## Finding your way around

Open `/api/docs` on your instance. It is the full reference, generated from the server
itself, so it is never out of date: every endpoint, what it takes, what it answers, and
which role it needs. You can try calls from there once you have a token. The same
document is served as `/api/openapi.json` for tools that generate clients from it.

Live updates go over WebSockets, which that page cannot show. They are described in the
repository under `docs/realtime-api.md`.

## Getting a token

Open **Settings** from your avatar, then **API tokens**. Give the token a name that says
what it is for, and optionally a lifetime. The token is shown once, right after it is
created. Copy it then; the list afterwards shows only the name, when it was last used and
when it expires.

A token acts as you: it can see the maps you can see and do what your role on each of
them allows, and nothing more. It cannot make or remove other tokens, so if one leaks you
revoke it from the same page and the leak stops there.

## Calling it

Send the token as a bearer header:

```
curl -H "Authorization: Bearer wst_..." https://your.instance/api/maps
```

That lists your maps. Map actions are `POST` requests under `/api/maps/{id}/` with a JSON
body that names the map again; the reference shows the body for each one.

## Expect it to change

WormholeSystems is in beta, and the API follows the map screen. Endpoints get renamed and
bodies gain and lose fields without notice. If a script stops working after an update,
`/api/docs` is where to look for what moved.

## When a token stops working

A request with an unknown, revoked or expired token is refused outright with `401`, never
quietly treated as a visitor. If a script starts getting that, look at the tokens page: the
one it used is gone or past its date.
