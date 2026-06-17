---
title: Setting waypoints
---

# Setting waypoints

Once you have a route, you can push it straight to your in-game autopilot — no clicking around the star map.

## What you need

The **`esi-ui.write_waypoint.v1`** ESI scope. Grant it from the **Scopes** page (see [Connecting your character](/documentation/getting-started/connecting-your-character)). Without it, the app can still _show_ you a route but can't set it in-game.

## How a waypoint is added

When the app sets a waypoint it has three controls, matching EVE's own autopilot options:

- **Clear other waypoints** — wipe the existing route and make this the only destination, or leave the current route in place and add to it.
- **Add to the beginning** — insert this system as the _next_ stop (you fly through it first), instead of appending it to the _end_ of the queue.

So you can set a fresh destination, or build up a multi-stop route by adding waypoints one at a time.

## Bulk waypoints

Need a whole fleet pointed at the same place? **Bulk** waypoint setting sends the same destination to **all of your connected characters that are currently online and have the waypoint scope** — every alt routed together in one action.
