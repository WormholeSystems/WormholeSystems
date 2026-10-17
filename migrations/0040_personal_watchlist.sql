-- Systems one user watches on a map, for themselves alone. Kept apart from the map's
-- shared `map_watchlist`, so the map's commands, history and export never see them.
create table map_personal_watchlist (
    id              bigint generated always as identity primary key,
    map_id          bigint not null references maps (id) on delete cascade,
    user_id         bigint not null references users (id) on delete cascade,
    solar_system_id bigint not null references solar_systems (id),
    is_pinned       boolean not null default false,
    created_at      timestamptz not null default now(),

    unique (map_id, user_id, solar_system_id)
);
