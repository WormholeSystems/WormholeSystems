---
title: K-Space connections
---

# K-Space connections

Bookmarks for connections into **known space** (high-, low-, and null-sec) use this format:

```
[NUMBER] [CLASS] [SIGNATURE] [SYSTEM] [REGION]
```

## The parts

- **Number** — the system's sequential number, the same [path number](/documentation/bookmarking/why-this-system) used for wormholes. It comes first because it _is_ the route to the hole.
- **Class** — the security band of the destination:
  | Prefix | Meaning |
  | --- | --- |
  | `HS` | High Sec |
  | `LS` | Low Sec |
  | `NS` | Null Sec |
- **Signature** — the signature **letters only**, no numbers (e.g. `ABC`, not `ABC-123`).
- **System** — the destination system name.
- **Region** — the destination region.

## Examples

```
21 HS ABC Amarr Domain
283 NS XYZ 1DQ1-A Delve
```

So `283 NS XYZ 1DQ1-A Delve` reads as: follow `2` → `28` → `283`, where the `283` is a null-sec exit (signature `XYZ`) into 1DQ1-A in Delve. The number keeps the hole grouped with the rest of its system and tells you how to get there; the class, system, and region tell you where you come out.
