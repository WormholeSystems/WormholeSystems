-- A map now holds up to two rally points (a form-up spot and the fight itself, say), and
-- setting one more replaces the oldest. That needs to know when each was set, so the flag
-- becomes a timestamp. `is_rally` stays as a column generated from it, so everything that
-- only asks "is this a rally point" reads the same as before. The cap lives in the app
-- (`solar_system::MAX_RALLY_POINTS`): a partial index can say one, not two.
alter table map_solar_systems add column rallied_at timestamptz;
update map_solar_systems set rallied_at = now() where is_rally;

drop index map_solar_systems_one_rally;
alter table map_solar_systems drop constraint map_solar_systems_ghost_unmarked;
alter table map_solar_systems drop column is_rally;
alter table map_solar_systems
    add column is_rally boolean not null generated always as (rallied_at is not null) stored;

alter table map_solar_systems add constraint map_solar_systems_ghost_unmarked
    check (solar_system_id is not null or not (is_home or rallied_at is not null or is_pinned));
