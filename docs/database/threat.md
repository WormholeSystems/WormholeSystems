# Threat

Tables behind [threat analysis](../features/threat.md).

## `killmails`

Minimal locally-ingested killmails from zKillboard's R2Z2 stream.

| Column | Type | Notes |
|--------|------|-------|
| `id` | pk (bigint) | the killmail id |
| `hash` | text | killmail hash |
| `solar_system_id` | bigint | where it happened (no FK: any system id) |
| `time` | timestamptz | killmail time; retention 730 days |
| `orgs` | jsonb | participating orgs, deduped: `[{"id", "kind"}]`, alliance preferred over corporation |
| `ingest_seq` | bigint, null | order the live stream stored it in (`killmails_ingest_seq`); null for archive imports |

Index on `(solar_system_id, time)` for the analysis window scan, and a BRIN index on
`ingest_seq` so the name resolver finds what is new since its last pass.

## `zkb_state`

Single-row cursor into the R2Z2 sequence stream (`id boolean pk default true`,
`sequence_id bigint`), and `named_seq bigint` (default 0): the last `ingest_seq` the
name resolver is done with.

## `wormhole_systems` additions

| Column | Type | Notes |
|--------|------|-------|
| `threat_level` | text, default `unknown` | `unknown` / `high` / `critical` |
| `threat_analyzed_at` | timestamptz, null | when the daily batch last ran |

## `wormhole_system_threats`

The top organisations per system, fully replaced by each analysis run.

| Column | Type | Notes |
|--------|------|-------|
| `id` | pk identity | |
| `solar_system_id` | fk wormhole_systems | cascade |
| `entity_id` | bigint | alliance or corporation id |
| `entity_type` | text | `alliance` / `corporation` |
| `name` | text | resolved at analysis time |
| `kills` | int | kills in the 90-day window |

Unique `(solar_system_id, entity_type, entity_id)`.
