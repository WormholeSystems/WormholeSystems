---
title: Threat analysis
---

# Threat analysis

The **Wormhole Threat Analysis** panel gives each wormhole system a threat rating based on how much fighting has happened there recently — so you can judge how dangerous a system is before you commit to it. Unlike the live [killmail feed](/documentation/intel/killmails), this is a longer-term picture of who operates in a system.

## How the rating is calculated

For the wormhole system you have selected, the app looks at **killmails in that system over the last 90 days** and counts the total. That total maps to a level:

| Level        | Kills in the system (last 90 days) |
| ------------ | ---------------------------------- |
| **Critical** | 50 or more                         |
| **High**     | 15 or more                         |
| **Unknown**  | fewer than 15, or no data          |

It also works out **which organisations** are behind that activity. It tallies kills per corporation and alliance — counting groups that appear as attacker _or_ victim — keeps those that were active on several different days (so a single busy day doesn't make a group look entrenched), and surfaces the **top organisations** by kill count.

## What you see

- A colour-coded **threat badge** — red for Critical, orange for High, grey for Unknown.
- The **top organisations** active in the system, each with its kill count and links out to zKillboard.
- An **"analysed … ago"** timestamp telling you how fresh the assessment is.

On the map itself, systems can show a coloured **ring** — red for Critical, orange for High. Toggle this with **Show Threat Level** in **Map settings → Preferences** ("Display a colored ring around wormhole systems based on killmail activity"); it's on by default.

## How fresh it is

The underlying killmails stream in from zKillboard in real time, but the threat ratings are **recomputed in a daily batch** — so the level and the organisation breakdown reflect the last analysis run, not the last five minutes. The "analysed … ago" stamp tells you exactly how current it is. For up-to-the-minute activity, watch the [killmail feed](/documentation/intel/killmails) instead.

> Threat analysis describes a _system_, not a specific gang being online right now. A Critical rating means the system sees a lot of PvP over time — treat it as "expect trouble here", then use live killmails and [character tracking](/documentation/tracking/character-presence) to read the current moment.
