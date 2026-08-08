---
title: Wormhole connections
---

# Wormhole connections

Bookmarks for connections into **wormhole space** use this format:

```
[NUMBER] [SIGNATURE] [CLASS]
```

## The parts

- **Number** — the hole's [path number](/documentation/bookmarking/why-this-system). Within any system, holes are numbered sequentially, and each one _extends its parent's number_: home's holes are `1`, `2`, `3`; the holes inside the `2` are `21`, `22`, `23`; the holes inside the `28` are `281`, `282`, `283`. The number is therefore both a unique handle _and_ the route to the hole.
- **Signature** — the signature **letters only**, no numbers (e.g. `ABC`).
- **Class** — the wormhole class of the destination, e.g. `C3`, `C5`.

## The return hole

The hole you came in through is named with just a `*` and the number:

```
*[NUMBER]
```

This makes it instantly clear which hole you're currently sitting next to, even in a freshly-scanned system with no other connections yet.

## Examples

```
*1
11 ABC C3
12 XYZ C5
13 DEF C2
14 NS QWE 1DQ1-A Delve
```

This is the bookmark list _inside the `1`_: `*1` is the return hole back the way you came, and `11`–`14` are the holes leading deeper, each extending the `1`. Notice the last line — a connection out to **known space** keeps its path number and then follows the [K-Space format](/documentation/bookmarking/k-space-connections) — number, class, signature, system, region.

> You rarely type these by hand — the mapper builds them for you. See [Let the mapper name it](/documentation/bookmarking/let-the-mapper-name-it).

Prefer letters to numbers? A manager can switch the whole map to an [alternative alias convention](/documentation/bookmarking/alternative-alias-convention).
