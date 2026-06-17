---
title: Pasting scan results
---

# Pasting scan results

The fastest way to record signatures is to paste them straight from the in-game scanner.

## Steps

1. Open the **Probe Scanner** in EVE and select all results (`Ctrl+A`).
2. Copy them (`Ctrl+C`).
3. In the **Signatures** panel, paste (`Ctrl+V`).

The app reads the tab-separated rows EVE copies (signature ID, group, category, and type) and reconciles them against the system's current list. Before anything is saved you see a preview of what will change:

- **New** signatures that will be added.
- **Updated** signatures whose category or type the paste has resolved.
- **Missing** signatures — ones already on the map that weren't in your paste.

You stay in control of the missing ones: confirm to delete the stale signatures, or keep them. Because the paste reconciles rather than blindly replacing, you can re-paste a fresh scan any time and the list stays current.

> Signatures only need their ID to be recorded — the six-character code is enough. You can resolve the type later, by hand or by pasting a more complete scan once you've probed it down. Temporary event sites that aren't in the database can be stored under a free-text name.
