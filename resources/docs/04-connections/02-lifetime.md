---
title: Lifetime
---

# Lifetime

A wormhole's **lifetime** is how much time it has left before it collapses on its own.

| Status                | Time remaining                                                                               |
| --------------------- | -------------------------------------------------------------------------------------------- |
| **Healthy**           | More than ~4 hours. Safe to rely on.                                                         |
| **End of life (EOL)** | Under ~4 hours. The hole has started to destabilise and may collapse soon — cross with care. |
| **Critical**          | Under ~1 hour. Treat collapse as imminent; don't get stranded on the wrong side.             |

## Set it, and let it age

You can set a connection's lifetime by hand, but the app also **ages it automatically**. Based on how long the hole has been on the map and what kind of wormhole it is, a connection moves toward EOL and then critical as it approaches its natural maximum age — and once a hole is marked EOL, it's treated as critical a few hours later.

> Different wormhole types live different lengths of time, so the app uses the hole's type to decide when to age it. A standard hole and a C6 static don't reach end-of-life at the same wall-clock age.

Lifetime is independent of [mass](/documentation/connections/mass) — a hole can be fresh on mass and still end-of-life on time. Autopilot can be told how aged a hole it's willing to use; see [Wormhole filters](/documentation/autopilot-and-routing/wormhole-filters).
