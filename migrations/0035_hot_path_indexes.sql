-- Indexes for the filters the hot paths and the routine deletes actually use, plus what the
-- daily retention sweep needs to find old rows without scanning for them.

-- Every map read lists its connections by map, and so do the stale sweep, the map list's
-- counts and the alert chain. Without this each of those scanned every connection on the
-- instance. The two endpoint indexes are for the cascade when a placement is removed.
create index map_connections_map on map_connections (map_id);
create index map_connections_from_system on map_connections (from_system);
create index map_connections_to_system on map_connections (to_system);

-- Every ingested killmail asks which maps hold its system; the unique index on
-- (map_id, solar_system_id) leads with the map and cannot answer that. Ghosts have no
-- system, hence partial.
create index map_solar_systems_solar_system on map_solar_systems (solar_system_id)
    where solar_system_id is not null;
-- Removing a placement cascades to the ghosts hanging off it.
create index map_solar_systems_hangs_off on map_solar_systems (hangs_off_id)
    where hangs_off_id is not null;

-- The history purge deletes old rows one by one as far as the foreign keys go: each
-- deleted event nulls its children's `parent_id` and any map whose head it was. Both
-- lookups were full scans per deleted row.
create index map_events_parent on map_events (parent_id) where parent_id is not null;
create index maps_head_event on maps (head_event_id) where head_event_id is not null;
-- The history endpoint pages by id, newest first.
create index map_events_map_id on map_events (map_id, id desc);

-- Loading a character's token, on every poll.
create index esi_tokens_character on esi_tokens (character_id);

-- "Which maps does this user share their position with", on every jump and every status
-- change. The primary key leads with the map.
create index map_user_settings_tracking on map_user_settings (user_id) where tracking_allowed;

-- The gate check on every observed system change.
create index stargates_route on stargates (solar_system_id, destination_system_id);

-- A btree reads both ways, so `killmails_system_time` already serves the newest-first
-- card; this one was a second copy of it paid for on every insert.
drop index killmails_system_recent;

-- The retention sweep deletes alert history by age across all maps; the existing index
-- leads with the map.
create index map_alert_events_age on map_alert_events (created_at);
-- And expired sessions.
create index sessions_expires_at on sessions (expires_at);
