-- The order the live stream stored killmails in, so the name resolver can carry on after
-- the last one it looked at instead of rescanning a week of everyone's kills every minute.
-- The killmail id will not do: zKillboard delivers a late kill well after newer ids.
-- Null for archive imports, which name their own.
create sequence killmails_ingest_seq;
alter table killmails add column ingest_seq bigint;

-- Assigned in insert order, so a BRIN index stays a few pages and still finds the tail.
create index killmails_ingest_order on killmails using brin (ingest_seq);

-- How far the resolver has got, next to the stream's own cursor.
alter table zkb_state add column named_seq bigint not null default 0;
