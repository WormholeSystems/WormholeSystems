---
title: Connecting your character
---

# Connecting your character

The app talks to EVE Online through **ESI**, CCP's official API. To unlock live features you grant a set of _scopes_ — permissions that let the app read certain data or perform certain actions on your behalf. Manage them any time from the **Scopes** link in the header (or grant them from within a map when it needs them).

## The scopes the app uses

| Scope                            | What it enables                                         |
| -------------------------------- | ------------------------------------------------------- |
| `esi-location.read_location.v1`  | Your current solar system — the basis of live tracking. |
| `esi-location.read_ship_type.v1` | Your current ship's type and name.                      |
| `esi-location.read_online.v1`    | Your online status.                                     |
| `esi-ui.write_waypoint.v1`       | Setting in-game autopilot waypoints from the map.       |
| `publicData`                     | Basic public character information.                     |

The first three power [character tracking](/documentation/tracking/character-presence); the waypoint scope powers [setting waypoints](/documentation/autopilot-and-routing/setting-waypoints). If a scope is missing, the header shows how many are outstanding so you can grant them in one place.

## Multiple characters

You can connect several characters and switch your **active character** at any time. Tracking and waypoint actions apply to the character you choose, and [bulk waypoints](/documentation/autopilot-and-routing/setting-waypoints) can target several of your characters at once.

> Scopes are under your control. Revoke them at any time from the Scopes page (which removes the app's tokens for that character) or from EVE's third-party application settings.
