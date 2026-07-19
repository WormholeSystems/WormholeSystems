---
title: Alternative alias convention
---

# Alternative alias convention

The [standard convention](/documentation/bookmarking/why-this-system) numbers holes: `1, 2, 3`, then `21, 22, 23` inside the `2`. Some groups prefer letters instead. A manager can switch the whole map to an **alphabetical** scheme in **Map settings → Mapping → Bookmark Format**.

This only changes what the mapper **suggests** going forward — switching schemes never renames anything already on the map, and the two schemes can coexist on a map that's mid-switch.

## The convention

- **Wormhole children extend the prefix with a letter**: `A` → `AA, AB, AC…`; `AB` → `ABA, ABB, ABC…`.
- **`H`, `L`, `N` and `P` are reserved** and never used as a wormhole branch letter — the sequence skips them: `…G, I, J, K, M, O, Q…`.
- **K-space exits use the reserved letter for their security class, plus a per-type index** off the parent:

  | Class        | Pattern           | Example |
  | ------------ | ------------------ | ------- |
  | High-sec     | `<parent>H<n>`      | `AH1`, `AH2` |
  | Low-sec      | `<parent>L<n>`      | `ACL1` |
  | Null-sec     | `<parent>N<n>`      | `BN2` |
  | Pochven      | `<parent>P<n>`      | `AP1` |

- **An unaliased home's direct holes** get the empty-prefix case — `A, B, C…` — mirroring the numeric scheme's `1, 2, 3…`. A manually-named root (e.g. naming your home `A`) works too, since a suggestion just extends whatever prefix already exists.
- **K-space exits can branch further**: a wormhole found off `AH1` suggests `AH1A`, exactly like any other wormhole child.

## Example

The bookmark list inside an alphabetical `A`:

```
*A
AA ABC C3
AB XYZ C5
AH1 HS QWE 1DQ1-A Delve
AH2 HS RST J5A-IX Curse
```

`AH1` and `AH2` are two separate high-sec exits off `A` — the high-sec counter is independent per parent, so it doesn't collide with the wormhole letters `AA`/`AB`.

## Known limitation

A single system with more than 22 direct wormhole children (there are only so many non-reserved letters) will start repeating the last suggestion, the same way the numeric scheme's suggestions become ambiguous past 9 children of one system. Neither case is common enough to warrant a two-letter or multi-digit escape hatch.

## Switching schemes

Only **managers** can change the scheme, in the same **Bookmark Format** card used to [customize the bookmark template](/documentation/bookmarking/customizing-the-format). The change is shared by everyone on the map, propagating live like the rest of that card's settings.
