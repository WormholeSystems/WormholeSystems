-- When somebody last pasted a scan into this system, so the signatures panel can say how
-- fresh the list is. Only a paste sets it: a hand edit is not a rescan. It lives on the
-- placement, so removing the system forgets it along with the signatures.
alter table map_solar_systems add column signatures_pasted_at timestamptz;
