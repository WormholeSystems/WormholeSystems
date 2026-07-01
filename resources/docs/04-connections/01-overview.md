---
title: Overview
---

# Connections

A **connection** is a link between two systems on the map — almost always a wormhole. Each one carries the status your fleet needs to decide whether it's safe to take a ship through, and the [routing engine](/documentation/autopilot-and-routing/how-routing-works) uses the same data when it plans paths.

Every connection tracks three things:

- **Lifetime** — how much time the hole has left before it collapses.
- **Mass** — how much total ship mass it can still pass.
- **Ship size** — the largest hull it will allow through.

A connection also remembers when it was first connected and when its lifetime last changed, and it can hold the wormhole type and the [signatures](/documentation/signatures/linking-wormholes) linked to it.

You maintain mass and ship size yourself as the hole is used; lifetime is something you can set and that the app also ages automatically over time. Keeping these honest matters — they feed straight into routing, and the [wormhole filters](/documentation/autopilot-and-routing/wormhole-filters) use them to decide which holes are safe to route through.

A connection can also be marked as a [stargate instead of a wormhole](/documentation/connections/connection-type), and flagged to [preserve its mass](/documentation/connections/preserve-mass) so the fleet keeps heavy ships off it.

The next pages explain each status and its levels.
