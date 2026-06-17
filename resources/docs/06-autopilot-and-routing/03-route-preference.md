---
title: Route preference
---

# Route preference

Route preference controls **how each jump is scored** during the search. The router always returns the lowest-total-cost path — preference decides what "cost" means.

| Preference      | Help text | What it optimises for                          |
| --------------- | --------- | ---------------------------------------------- |
| **Shorter**     | Min jumps | Fewest jumps, ignoring security entirely.      |
| **Safer**       | High-sec  | Prefers high-sec; penalises low- and null-sec. |
| **Less Secure** | Low-sec   | Prefers low-sec; penalises high- and null-sec. |

## Shorter

Every jump costs exactly the same, so the router simply finds the path with the fewest jumps. Security status is not considered. This is the default.

## Safer and Less Secure

These modes give each jump a cost based on the **security of the system you're jumping into**, then find the cheapest path. Cheaper systems are favoured, more expensive ones are avoided:

- **Safer** makes high-sec cheap and low/null-sec expensive.
- **Less Secure** makes low-sec cheap and high/null-sec expensive.

In **both** modes, **null-sec is penalised the hardest** — twice as much as the merely-disfavoured security band — so neither mode willingly routes through null unless there's no alternative.

## The security penalty

When preference is **Safer** or **Less Secure**, a **Security Penalty** slider (0–100, default 50, in steps of 5) sets _how strongly_ the disfavoured space is avoided.

The penalty grows **exponentially**, not linearly — the cost multiplier is roughly `e^(0.15 × penalty)`:

| Penalty          | Effect                                                                                   |
| ---------------- | ---------------------------------------------------------------------------------------- |
| **0**            | No penalty — every jump costs the same and the route is effectively shortest-path again. |
| **50** (default) | Strong avoidance — the router will take many extra jumps to stay in favoured space.      |
| **100**          | Near-absolute — disfavoured systems are avoided unless literally the only way through.   |

> Because the curve is exponential, the difference between 40 and 60 is far larger than the numbers suggest. If a "Safer" route still dips through low-sec, raise the penalty; if it's taking absurd detours, lower it.

The slider is hidden for **Shorter**, where it has no effect.
