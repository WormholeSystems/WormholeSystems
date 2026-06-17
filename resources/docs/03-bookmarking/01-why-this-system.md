---
title: Why we bookmark this way
---

# Why we bookmark this way

Bookmarking in wormhole space is one of those things everyone does slightly differently — and those small differences add up to wasted time, rescans, and confusion mid-fight. This is an **opinionated** convention. It isn't the only way to name bookmarks, but it's the one we've found to be the most **declarative and efficient**, and the value comes from _everyone using the same one_.

The whole convention rests on two rules:

1. **Every resolved signature gets a bookmark immediately.**
2. **Bookmarks and mapper entries must always match.**

If those two hold, anyone can drop into the chain and read it instantly.

## The name is the route

The real power of the convention is that a wormhole's number **is its route description**. Aliases are built up hole by hole as you map outward: the system you reach through home's hole `2` becomes the `2`; a hole you scan inside it becomes the `28`; the system beyond _that_ is the `283`. Each number is simply its parent's number with the next hole appended.

So the name spells out the path. To reach the `283` you jump the `2`, then the `28`, then the `283` — in that order. When someone on comms says _"I'm in the 283"_, you don't have to open the map to work out where that is: open your in-game bookmarks and follow `2` → `28` → `283`, one hole at a time. **The structure of the entire chain is encoded in every single name** — the bookmark list alone is enough to navigate.

## Why it works so well

- **Declarative — the name tells you everything.** A bookmark like `12 XYZ C5` or `21 HS ABC Amarr Domain` says where the hole goes, what class it is, and which signature it is, without opening the scanner or the map. You read the destination straight off the label.
- **It doubles as turn-by-turn directions.** As above — the number is the path, so you can follow bookmarks to any system without touching the map.
- **It sorts itself.** Because the number (or class) comes first, the in-game bookmark list falls into a logical order on its own, and a system's holes group together right under it. The hole you want is where you expect it, every time.
- **Short, stable handles for comms.** Numbered holes give everyone a quick way to talk about a hole — "tackle on the 283", "collapse the 28" — that doesn't change as the chain grows.
- **You always know where you stand.** The `*` return marker shows at a glance which hole you came in through, even in a system with no other connections yet.
- **No wasted characters.** Signatures are letters-only — the three letters already identify the hole uniquely within a system, so the digits add nothing.
- **It's generated for you.** The mapper builds these names from the chain (see [Let the mapper name it](/documentation/bookmarking/let-the-mapper-name-it)), so there are no typos and the bookmark always matches the map.

The pages that follow give the exact formats for [known space](/documentation/bookmarking/k-space-connections) and [wormhole space](/documentation/bookmarking/wormhole-connections), how the numbers translate into [comms](/documentation/bookmarking/talking-about-holes), and how to [keep the chain healthy](/documentation/bookmarking/keep-the-chain-healthy) for everyone.
