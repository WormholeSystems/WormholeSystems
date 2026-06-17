---
title: How routing works
---

# How routing works

The router finds the best path between two systems across a single graph that combines three kinds of links:

- **Stargates** — the static New Eden gate network.
- **Your map's wormhole connections** — the chain you've scanned.
- **EVE Scout connections** — optional public Thera/Turnur holes (see [EVE Scout](/documentation/autopilot-and-routing/eve-scout)).

It runs a **weighted shortest-path search** (Dijkstra). Every link is an edge, and each edge has a _cost_. The router returns the path with the lowest total cost — which, depending on your settings, means the fewest jumps or the safest trip.

## Two things you control

1. **[Wormhole filters](/documentation/autopilot-and-routing/wormhole-filters)** decide _which_ wormholes the router is allowed to use at all — anything too far gone is removed from the graph before the search runs. Stargates are always allowed.
2. **[Route preference](/documentation/autopilot-and-routing/route-preference)** decides _which allowed path_ is best, by changing how each edge's cost is calculated.

## Settings are yours alone

All routing settings are stored **per map, per user**. Tuning your own filters or preference never changes how the chain or routes look for anyone else.

## Defaults

Out of the box, routing is tuned to "just get me there":

| Setting                  | Default                             |
| ------------------------ | ----------------------------------- |
| Route preference         | **Shorter** (fewest jumps)          |
| Wormhole lifetime filter | **Critical** (allow all)            |
| Wormhole mass filter     | **Reduced** (allow fresh + reduced) |
| Use EVE Scout            | **Off**                             |
| Security penalty         | **50**                              |

> The special system **Zarzakh** is never routed through.
