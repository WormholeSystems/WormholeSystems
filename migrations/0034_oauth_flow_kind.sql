-- Which handshake minted a state, so a callback only accepts its own. Without it the
-- Discord callback would take a state the EVE login started, and the other way round.
-- Rows only live ten minutes, so the few in flight are dropped rather than guessed at:
-- whoever was mid-login starts over.
delete from oauth_login_flows;

alter table oauth_login_flows
    add column kind text not null check (kind in ('eve_login', 'discord_link'));
