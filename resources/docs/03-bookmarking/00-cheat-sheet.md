---
title: Cheat sheet
---

# Bookmarking cheat sheet

The whole convention on one page. The details — and the _why_ — live in the pages that follow; this is the quick reference to keep open while you scan.

## The two rules

1. **Every resolved signature gets a bookmark immediately.**
2. **Bookmarks and mapper entries must always match.**

If both hold, anyone can drop into the chain and read it instantly. See [why we bookmark this way](/documentation/bookmarking/why-this-system).

## Formats

**Wormhole connections** — see [wormhole connections](/documentation/bookmarking/wormhole-connections):

```
[NUMBER] [SIGNATURE] [CLASS]
```

**K-Space connections** (high-, low-, null-sec) — see [k-space connections](/documentation/bookmarking/k-space-connections):

```
[NUMBER] [CLASS] [SIGNATURE] [SYSTEM] [REGION]
```

**The return hole** — the hole you came in through:

```
*[NUMBER]
```

## The parts

- **Number** — the system's sequential [path number](/documentation/bookmarking/why-this-system). Holes are numbered per system and each one _extends its parent_: home's holes are `1`, `2`, `3`; the holes inside the `2` are `21`, `22`, `23`. The number is both a unique handle and the route to the hole. Prefer letters instead? See the [alternative alias convention](/documentation/bookmarking/alternative-alias-convention).
- **Signature** — the **letters only**, no numbers (`ABC`, not `ABC-123`).
- **Class** — `C1`–`C6` for wormholes; `HS` / `LS` / `NS` for known space.

## Examples

This is the bookmark list _inside the `1`_:

```
*1
11 ABC C3
12 XYZ C5
13 DEF C2
14 NS QWE 1DQ1-A Delve
```

`*1` is the return hole back the way you came; `11`–`14` lead deeper. The last line is a known-space exit — it keeps its path number, then follows the K-Space format.

## Let the mapper name it

You rarely type these by hand — see [let the mapper name it](/documentation/bookmarking/let-the-mapper-name-it).

- **Copy name** — right-click a connection, open **Copy name**, pick the side you're in, paste in-game.
- **Copy Bookmark** — turn it on in **Map settings → Mapping** to copy the name automatically when you record a jump.
- **Prompt for Signature** — also in **Map settings → Mapping**; the mapper asks which signature you jumped, turning logging into one click.

## Talking about holes

The number is also how you call your position — see [talking about holes](/documentation/bookmarking/talking-about-holes).

- **"on the [number]"** — at that hole on the **parent** side, about to jump in.
- **"on the [number] return"** — on the **far** side, at the hole that jumps back.

## Keep the chain healthy

- Bookmark every resolved signature **immediately**.
- Post every signature you scan — it takes ~10 seconds and saves the next person a rescan.
- Add all connections to the [shared map](/documentation/getting-started/overview) so routing works for everyone.

More in [keep the chain healthy](/documentation/bookmarking/keep-the-chain-healthy).
