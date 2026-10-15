# Realtime map updates

Several people view and edit the same map at once. When one of them changes something —
places a system, marks a connection EOL, links a signature — the others should see it
without reloading. This doc specifies the **event system** behind that: how a change
becomes an event, how it reaches a viewer, and what the viewer does with it.

The application layer it sits above is [Map actions](./maps.md); the data it reflects is
the [mapping](../database/mapping.md) tables.

## Decision: an in-process bus, not Postgres `LISTEN/NOTIFY` or a broker

Mapping runs as a **single server process** and is expected to for the foreseeable future.
That makes the simplest option also the best one: an **in-process publish/subscribe bus**
([`MapHub`](../../src/realtime/map_hub.rs)) built on `tokio::broadcast`, one channel per `map_id`.

Why not the alternatives:

- **Postgres `LISTEN/NOTIFY`** would buy cross-instance delivery we don't need, at the cost
  of an 8 KB text payload limit and — the real dealbreaker — routing logic in SQL. The
  upcoming presence feature wants to notify "every user with access to map X that a tracked
  character moved"; access resolution is Rust logic over several tables, awkward to express
  in a trigger/`NOTIFY`. In-process, the producer just calls it.
- **A broker (Redis/NATS)** is infra we'd add only to scale past one process — premature.

If mapping ever needs to scale horizontally, the bus is the seam to swap: keep `MapEvent`,
back `MapHub` with `NOTIFY` or a broker. Nothing else changes.

## `MapEvent`

A typed enum, routed by `map_id`. It carries **what changed (ids), not the new data** —
consumers refetch the affected slice through the read actions. This "notify-then-refetch"
keeps payloads tiny and means there's exactly one source of truth (the DB read path), never
a half-applied push diff.

| Variant | Carries | Emitted when |
|---------|---------|--------------|
| `MapUpdated` | `map_id` | name / description / image changed |
| `SystemAdded` / `SystemMoved` / `SystemRemoved` | `map_id`, `map_solar_system_id` | placement changes |
| `ConnectionChanged` | `map_id`, `connection_id` | connection added / removed / state changed (incl. trigger-driven sync) |
| `SignatureChanged` | `map_id`, `solar_system_id` | a signature added / edited / linked / unlinked / removed |
| `AccessChanged` | `map_id` | a grant or role changed |
| `WatchlistChanged` | `map_id` | a watchlist entry added / pinned / removed |
| `IgnoredSystemsChanged` | `map_id` | the map's ignore list changed |

Coarse on purpose: each variant names a slice the client re-reads. Finer payloads (e.g. the
changed row inline) are an additive change later if refetch volume warrants it.

## `MapHub`

A cheaply-cloneable handle (an `Arc<Mutex<HashMap<map_id, broadcast::Sender>>>` inside) held
in app state.

- `subscribe(map_id) -> Subscription` — one per connected viewer; lazily creates the map's
  channel, and dropping the last subscription removes it, so idle maps don't leak. The map
  socket only subscribes once the reader's access is checked, so probing ids leaves nothing
  behind.
- `publish(event)` — routes by `event.map_id()`; a no-op if nobody is watching that map.

A receiver that falls behind the channel depth gets `Lagged`; the WebSocket layer treats that
as "you're behind" and triggers a full `get_map` rather than trying to replay.

## Where events come from — and the trigger interaction

The action functions in `src/maps/` stay **pure** (`fn(pool, actor, cmd)`, no realtime
dependency — they're driven straight from tests). Publishing happens at the **server-function
boundary** that calls them: after a mutation commits, the handler publishes the matching
`MapEvent`. Background producers (the future tracking poller) publish to the same hub directly.

This composes with the connection↔signature **sync trigger**
([sync spec](../database/mapping.md#keeping-a-connection-and-its-signatures-consistent)):
because every trigger firing is *downstream of an action the server already published for*,
and because events are notify-then-refetch, the client's refetch picks up the trigger-applied
changes too. The trigger never needs to emit anything itself.

## Status & what's deferred

Built now: `MapEvent` + `MapHub` (the bus, unit-tested in `realtime/map_hub.rs`).

Built since: the **Axum WebSocket endpoint** (`src/api/ws.rs`), the **Svelte client**
subscription (any frame → refetch, `frontend/src/lib/ws.ts`), and the **`publish` calls**
at the API handler boundary (`src/api/handlers.rs`).

Still deferred:

- **presence** (`Presence { map_id, character_id }`) from the tracking poller — the case that
  most justifies the in-process design.
