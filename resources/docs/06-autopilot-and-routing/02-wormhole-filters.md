---
title: Wormhole filters
---

# Wormhole filters

Filters decide **which wormholes the router may use**. A connection that fails a filter is removed from the graph entirely before the search runs, so no route will ever send you through it. Stargates are never filtered.

There are two independent filters, and each one names the **riskiest** state you're willing to accept — picking a level also allows everything healthier than it.

## Lifetime filter

| Option           | Help text    | Wormholes allowed        |
| ---------------- | ------------ | ------------------------ |
| **Healthy Only** | &gt; 4 hours | healthy only             |
| **End of Life**  | &lt; 4 hours | healthy + EOL            |
| **Critical**     | &lt; 1 hour  | healthy + EOL + critical |

## Mass filter

| Option            | Help text | Wormholes allowed          |
| ----------------- | --------- | -------------------------- |
| **High Mass**     | &gt; 50%  | fresh only                 |
| **Reduced Mass**  | &lt; 50%  | fresh + reduced            |
| **Critical Mass** | &lt; 10%  | fresh + reduced + critical |

## How to think about it

- Set both filters to the **healthiest** option (Healthy Only / High Mass) when you're moving something you can't afford to lose — the router will only route through holes that are very unlikely to collapse.
- Loosen them toward **Critical** when you just need the shortest path and accept the risk.

The status values themselves are explained on the [connections](/documentation/connections/overview) pages. Filters only consider a connection's status — they don't change it.
