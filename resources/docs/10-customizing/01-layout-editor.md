---
title: Layout editor
---

# Layout editor

The map is built from **panels** arranged on a grid. The layout editor lets you arrange them to fit how you work.

## Panels

Three panels are always present — **Map**, **System Info**, and **Signatures**. The rest can be shown or hidden:

- Audits
- Ship History
- Characters
- Killmails
- Autopilot
- EVE Scout
- Wormhole Threat Analysis
- Raidable Skyhooks

## Editing

Enter **edit mode** to **drag** panels, **resize** them, and **show or hide** any of the optional ones. When you're done you can:

- **Save** — keep the arrangement.
- **Revert** — discard unsaved changes and go back to the last saved layout.
- **Reset** — return the current breakpoint to the built-in default.

## Copy & paste a layout

Spent time perfecting a layout and want it on another device, or to hand it to a corpmate? **Copy layout** packs your whole arrangement — _every_ breakpoint plus your hidden-panel choices — into a single text string and puts it on your clipboard (you'll see a "Layout copied to clipboard" confirmation).

Anyone can then use **Paste layout**: it reads that string from the clipboard, checks it's a valid layout, and applies it. A bad or unrelated clipboard value is rejected with an error rather than breaking your current layout.

A few things worth knowing:

- The string carries **all four breakpoints at once**, so pasting replaces your full responsive layout, not just the screen size you're on.
- It's just text — paste it into a message, a notes doc, or a pinned channel so your group can share a standard layout.
- Pasting changes your working layout but doesn't commit it — **Save** afterwards to keep it (or **Revert** to back out).

## Per-screen layouts

Layouts are saved per **breakpoint**, so each screen width gets its own arrangement:

| Breakpoint      | For                                | Columns |
| --------------- | ---------------------------------- | ------- |
| **Extra Small** | Phones                             | 1       |
| **Small**       | Phones & tablets (640px+)          | 2       |
| **Medium**      | Tablets & small desktops (1024px+) | 4       |
| **Large**       | Desktops & wide screens (1536px+)  | 10      |

Editing one breakpoint leaves the others untouched. Your layout is stored on your account (per map), so it follows you to any device.
