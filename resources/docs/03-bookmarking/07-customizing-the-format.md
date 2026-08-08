---
title: Customizing the format
---

# Customizing the bookmark format

> So. You've looked upon our godly, perfect, divinely-inspired [default bookmarking system](/documentation/bookmarking/why-this-system), honed over countless hours in the deep, and decided, for reasons known only to you, that it is _not quite right_. Bold. We respect it. Against our better judgement, here is how to bend the format to your will.

The [copy-name](/documentation/bookmarking/let-the-mapper-name-it) feature builds bookmarks from a **template** you control. Every map ships with the standard convention out of the box, but a manager can reshape it to match how the group likes to name holes.

The format is a **per-map setting shared by everyone** — one group, one convention. Only **managers** can change it, and the moment they save, everybody on the map copies names in the new format.

## Where to edit it

Open **Map settings → Mapping** and find the **Bookmark Format** card. There are two templates:

- **Wormhole systems** — used when the bookmarked system is a wormhole (`C1`–`C6`).
- **K-space systems** — used for high-, low-, and null-sec.
- **Return connections** — used when the bookmark leads back up the chain (`*` tag).

The mapper picks the right template automatically based on the system you're bookmarking.

## Tokens

A template is plain text with `{tokens}` in it. Each token is replaced with a value from the connection; anything else you type is kept verbatim. Click a token button to append it, or type your own text around them. A live **preview** shows a sample result as you go, and nothing is applied until you press **Save**.

| Token        | Becomes                                                                          | Example                |
| ------------ | -------------------------------------------------------------------------------- | ---------------------- |
| `{alias}`    | The system's alias / [path number](/documentation/bookmarking/why-this-system)   | `Home`, `1`, `21`      |
| `{sig}`      | The signature letters only, no digits                                            | `ABC`                  |
| `{class}`    | Wormhole class, or security band                                                 | `C3`, `HS`, `LS`, `NS` |
| `{name}`     | The solar system name                                                            | `J123456`, `Jita`      |
| `{region}`   | The region name                                                                  | `The Forge`            |
| `{occupier}` | The system's [occupier](/documentation/organizing-your-map/aliases-and-occupier) | `HK`, `CalNavy`        |
| `{size}`     | [Ship-size](/documentation/connections/ship-size) limit, when restrictive        | `SM`, `MD`, `XM`       |
| `{wh}`       | The [wormhole type](/documentation/bookmarking/talking-about-holes) code         | `K162`, `N110`         |
| `{mass}`     | [Mass](/documentation/connections/mass) state, when degraded                     | `reduced`, `crit`      |
| `{life}`     | [Lifetime](/documentation/connections/lifetime) state, when degraded             | `EOL`, `EOL!`          |

The defaults reproduce the standard convention:

```
Wormhole:  {alias} {sig} {class}
K-space:   {alias} {class} {sig} {name} {region}
```

## Tokens drop out when there's nothing to say

A token with no value simply disappears, and the extra spacing around it collapses — so a template never leaves a gap or a stray separator. This keeps names tidy when information is missing or a hole is still fresh:

- **`{size}`** only shows for **restrictive** holes — `SM` (frigate), `MD` (medium) and `XM` (very large only). A regular large-ship hole is the common case, so it shows nothing.
- **`{mass}`** stays empty while the hole is fresh and only appears once it's `reduced` or `crit` (critical).
- **`{life}`** stays empty while the hole is healthy and only appears at `EOL` (under ~4h) or `EOL!` (under ~1h).
- **`{wh}`, `{sig}`, `{size}`** are naturally blank on an unidentified hole or a plain stargate connection, so the same template works everywhere.

That's why `{mass}` uses `crit` while `{life}` uses `EOL`/`EOL!` — the two never render the same word, so a glance at the bookmark is never ambiguous.

## Examples

A tight wormhole template that surfaces danger only when it matters:

```
{alias} {sig} {class} {size} {mass} {life}
```

- Fresh, large C3 hole with signature `ABC` in your `21` branch → `21 ABC C3`
- Same hole once it's a frigate-sized, reduced, end-of-life connection → `21 ABC C3 SM reduced EOL`

Prefer the wormhole code up front? Nothing stops you:

```
{wh} {alias} {sig} {class}
```

→ `N110 Home ABC C3`

## Reverting

Each template has a **Reset to default** button that drops it back to the standard convention. As with any edit, it only takes effect once you **Save**.
