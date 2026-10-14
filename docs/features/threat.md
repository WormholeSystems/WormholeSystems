# Threat analysis

Kill-activity threat per wormhole system, ported from the legacy rules. Part of the
[feature specs](../README.md); tables in [database/](../database/).

## Ingest

`src/killmails.rs` polls zKillboard's R2Z2 sequence stream (`/ephemeral/sequence.json`,
then `/ephemeral/{seq}.json`) and persists a **minimal** row per killmail: id, hash,
solar system, time, and the participating organisations (victim + every attacker,
alliance preferred over corporation, each org at most once per killmail). The full ESI
payload is not stored. Retention is 730 days (daily purge). The cursor lives in
`zkb_state`: read once at boot, kept in memory after that, and written after every kill.

A backlog is read back to back, at most ten requests a second (R2Z2 bans past 15). Only
a 404, meaning nothing new yet, waits: 6 seconds, as zKillboard asks. Each stored kill is
offered to a queue of 1024 for the killmail alerts, which a worker evaluates in batches;
when it is full the kill is still stored, only its alerts are skipped, with a warning.

Both loops are gated behind `ZKB_LISTEN=1` so dev machines don't poll zKillboard by
default. All requests send a descriptive User-Agent; zKillboard rejects anonymous
clients with 403.

`wormholesystems killmails-backfill [days]` (default 30) imports EVE Ref's daily archives
(`https://data.everef.net/killmails/{year}/killmails-YYYY-MM-DD.tar.bz2`, extracted with
the system `tar`), newest day first, then runs the analysis once. A 404 means no
killmails were published for that day. Existing rows are left untouched, so it composes
with the live listener.

## Analysis (daily, full replacement)

Per wormhole system, over the last 90 days:

1. Count kills per organisation (an org scores one per killmail it appears in).
2. Drop orgs active on fewer than 5 distinct days.
3. Keep the top 10 by kills.
4. Sum their kills: `>= 50` → `critical`, `>= 15` → `high`, else `unknown`.

Results go to `wormhole_systems.threat_level` / `threat_analyzed_at` and the
`wormhole_system_threats` top list (names resolved from the local alliance/corporation
tables, falling back to ESI). The whole result set is replaced on every run.

Rules are covered by unit tests in `src/killmails.rs` and DB-backed tests in
`tests/threat.rs`.

## Surface

- `MapSystemView.threat_level` (wormhole systems only) drives the node's threat ring
  (`ring-threat-critical` red / `ring-threat-high` orange), gated by the per-user map
  setting `show_threat_level` and suppressed while the node is active.
- `GET /api/threat/{solar_system_id}` returns the level, analysis timestamp, and the top
  entities for the Threat card (badge, entity list with zKillboard links, freshness).
