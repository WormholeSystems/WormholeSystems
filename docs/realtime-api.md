# The realtime API

The HTTP API answers questions; the WebSockets say when the answers have changed. A
client that holds a map open subscribes to the map's stream, and on every frame refetches
the slice the frame names. The frames carry ids, never data: there is one source of truth
(the read endpoints) and the socket only says which part of it to read again.

The design behind this is in [`features/realtime.md`](./features/realtime.md). This page
is the wire contract for a client.

## Endpoints

Both are plain WebSocket upgrades on the same origin as the API. They are not in the
OpenAPI document, since it cannot describe a stream.

| Endpoint | Who may connect | Frames |
|----------|-----------------|--------|
| `GET /ws/map/{map_id}` | Anyone who may read the map: a grant at viewer or above, or the share it has been opened with (the `map_share_{map_id}` cookie the share route leaves behind). `404` otherwise, so a guesser learns nothing. | [`MapEvent`](#mapevent), one JSON object per text frame |
| `GET /ws/user` | A signed-in user. `401` otherwise. | [`UserEvent`](#userevent), one JSON object per text frame |

Authenticate the upgrade request the way you authenticate any other request: the session
cookie, or `Authorization: Bearer wst_...` with a personal access token.

A map socket keeps checking. Whenever the map's grants or sharing change, and once a
minute besides (a grant can run out, a character can leave the corporation it was granted
through), the server asks again whether the reader may still see the map. Once they may
not, it closes the socket with code `4403` and nothing more is sent. Do not reconnect on
`4403`: the upgrade would be refused with `404`. Code `1001` means the server is
restarting; reconnect on that as on any other close.

The user socket doubles as the activity heartbeat. The server pings every 30 seconds; any
frame from the client (the pong will do) counts as activity and keeps the user's
`last_active_at` fresh, which is what lets the tracking poller follow their characters. A
script that only wants events can ignore this; a client that wants tracking to work must
keep the socket open and answer the pings, which every WebSocket library does by default.

## Frames

Every frame is a JSON object with a `type` field in `snake_case` and the ids the event is
about. The schemas are in the OpenAPI document under `components.schemas.MapEvent` and
`components.schemas.UserEvent`, kept there so a generated client has them too.

### `MapEvent`

| `type` | Fields | Sent when | Read again |
|--------|--------|-----------|------------|
| `map_updated` | `map_id` | The map's name, description or image changed | `GET /api/maps/{id}` |
| `system_added` | `map_id`, `map_solar_system_id` | A system was placed | `GET /api/maps/{id}` |
| `system_moved` | `map_id`, `map_solar_system_id` | A placement moved | `GET /api/maps/{id}` |
| `system_removed` | `map_id`, `map_solar_system_id` | A placement was removed | `GET /api/maps/{id}` |
| `system_details_changed` | `map_id`, `map_solar_system_id` | Alias, status, occupier, home, rally, pinned or notes changed, or a scan was pasted into it (its `signatures_pasted_at` moved); the position did not | `GET /api/maps/{id}`, and the placement's details if you show notes |
| `connection_changed` | `map_id`, `connection_id` | A connection was added, removed, or had its mass, lifetime or size change, including the changes the signature sync trigger makes | `GET /api/maps/{id}` |
| `signature_changed` | `map_id`, `solar_system_id` | A signature in that system was added, edited, linked, unlinked or removed | `GET /api/maps/{id}/signatures` |
| `access_changed` | `map_id` | A grant was made, changed or revoked | `GET /api/maps/{id}/access`, and your own role |
| `watchlist_changed` | `map_id` | A watchlist entry was added, pinned or removed | `GET /api/maps/{id}/watchlist` |
| `ignored_systems_changed` | `map_id` | The map's ignore list changed | `GET /api/maps/{id}/ignored` |
| `characters_changed` | `map_id` | A tracked pilot moved, changed ship, or came on or offline | `GET /api/maps/{id}/characters` |
| `killmail_received` | `map_id` | Something died in one of the map's systems | `GET /api/maps/{id}/killmails` |
| `history_changed` | `map_id` | The command journal grew, or an undo or redo moved the map to another point in it. After an undo anything may have changed | `GET /api/maps/{id}/events`, and the whole map |

One more frame is not an event: `{"type":"lagged"}` means the client fell behind the
channel and frames were dropped. Treat it as "everything may have changed" and refetch the
map.

### `UserEvent`

| `type` | Fields | Sent when | Read again |
|--------|--------|-----------|------------|
| `character_status_changed` | `character_id` | One of your characters was seen somewhere else, in another ship, or came on or offline | `GET /api/me/status` |
| `server_status_changed` | | Tranquility went up, went down, or changed version. Sent to everyone connected | `GET /api/server-status` |
| `personal_watchlist_changed` | `map_id` | You changed your own watchlist on that map, from any tab | `GET /api/maps/{id}/watchlist/personal` |

## What a client should do

1. Fetch what it needs over HTTP.
2. Open the socket and, for each frame, refetch the slice in the last column above. Coalesce
   bursts: a paste of twenty signatures is twenty `signature_changed` frames for one system,
   and one refetch is enough.
3. On `lagged`, on reconnect, and on `history_changed`, refetch the map itself.

The frames say nothing a client could not learn by polling; they only say when.
