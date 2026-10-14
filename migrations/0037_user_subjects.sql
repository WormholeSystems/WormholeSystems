-- Who a user is, as far as a grant can tell: each of their characters, that character's
-- corporation, and its alliance. A grant names one of these, so "which maps can this user
-- see" is a join against this, written once here rather than as the same three-way union
-- in every query that asks.
create view user_subjects as
    select user_id, id as subject_id from characters where user_id is not null
    union all
    select user_id, corporation_id from characters where user_id is not null
    union all
    select user_id, alliance_id from characters
    where user_id is not null and alliance_id is not null;

-- A user's effective role on each map they can see: the highest of their live grants,
-- however they match it. One row per map and user; a user with no live grant has none.
-- Asked with both keys on every request, and Postgres pushes both through the grouping, so
-- the lookup stays on the characters (user_id) and map_access (map_id, subject_id) indexes.
create view map_user_roles as
    select a.map_id, s.user_id, max(a.role) as role
    from map_access_live a
    join user_subjects s on s.subject_id = a.subject_id
    group by a.map_id, s.user_id;
