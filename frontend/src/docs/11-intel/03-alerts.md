---
title: Discord alerts
---

# Discord alerts

An **alert** is a standing question about the chain that answers itself into Discord. Set
one up once and it speaks when the answer changes, instead of somebody watching a panel.

## What you can ask

| Kind           | Fires when                                                                     |
| -------------- | ------------------------------------------------------------------------------ |
| **Killmail**   | Something dies in the chain, filtered by who was involved and on which side.   |
| **Proximity**  | The chain comes within a set number of jumps of a system you named.            |
| **Jump range** | A capital of a given hull and jump-drive skill could reach a system you named. |

### Proximity with a starting point

By default a proximity alert measures from wherever the chain is nearest: any mapped system
within the jump limit of the target fires it. Give it a **starting point** and it measures
one route instead, from that system to the target through the chain, counting gate jumps
and treating wormholes as free. "Is Jita within five jumps of home" then stays about home,
not about whichever exit happened to land near Jita.

Such an alert fires when a placed system, or a freshly mapped wormhole, becomes part of a
route within the limit. Systems added elsewhere on the map do not re-fire it, and however
many changes report the same route it is said once; a new route to the target is a new
message.

## Where it goes

Destinations are named once per map and pointed at by as many alerts as you like. Pasting
the same webhook URL into four alerts is four chances to paste the wrong one, and rotating
it is four edits.

- **Channel webhook** — Discord's own channel setting, no application needed. This is the
  quickest way to start, and the only kind that works on an instance with no Discord bot.
- **Bot channel** — posts as the bot. Needs the instance to have a bot configured, and
  is set up with `/wh alert-channel` in the channel it should post to (see below). The alerts page can
  edit one afterwards, but not move it to another channel or ping someone new.
- **Direct message** — reaches one person. Needs the bot, and needs that person to have
  [linked their Discord account](/documentation/getting-started/connecting-your-character).

Roles are named the same way, so an alert can say who to ping without anyone reciting
`1189734502938472`.

> If the instance you are on has no Discord application configured, the alerts page says
> so. Webhooks still work; direct messages and bot posts do not.

## From Discord

The bot creates alerts too, so a scout does not have to leave the channel. Every command
starts with the destination, then the kind, then exactly the options that kind takes:

| Command                        | Options                                            |
| ------------------------------ | -------------------------------------------------- |
| `/wh alert-dm proximity`       | map, system, jumps, optional starting point        |
| `/wh alert-dm jump-range`      | map, system, ship, JDC level                       |
| `/wh alert-dm killmail`        | map, jumps                                         |
| `/wh alert-channel proximity`  | the same, plus who to mention and an optional role |
| `/wh alert-channel jump-range` | the same, plus who to mention and an optional role |
| `/wh alert-channel killmail`   | the same, plus who to mention and an optional role |

A direct-message alert needs only access to the map: it reaches nobody but you. A channel
alert posts into the channel the command was run in, so it takes a **manager** on the map
and the **Manage Channels** permission in that server. Mentioning a role also needs
**Manage Roles**, and mentioning everyone needs **Mention Everyone**. A role picked this
way is registered on the map under its Discord name, the same as one added on the alerts
page, and the alert appears there with the others: it can be renamed, edited, switched off,
or deleted like any other.

Alerts made from Discord get a name from what they watch, such as "Jita within 5 jumps"
or "Kills within 5 jumps of the chain". `/wh alerts list` shows the ones you created.

## When an alert stops

An alert disables itself rather than failing quietly, and says why: the creator unlinked
their Discord, their access to the map was revoked, the destination is gone, or Discord
rejected the delivery. Fix the cause and switch it back on.
