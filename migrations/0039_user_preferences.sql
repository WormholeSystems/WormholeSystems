-- How the site looks to one person, on every device and every map. A row is made on the
-- first save; a user without one gets the defaults below.
create table user_preferences (
    user_id      bigint primary key references users (id) on delete cascade,
    -- The game's icon on a wormhole effect badge, rather than its letter.
    effect_icons boolean not null default true,
    updated_at   timestamptz not null default now()
);
