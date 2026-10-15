-- When EVE refused to refresh this token (`invalid_grant`): access was revoked in EVE's
-- settings, or the refresh token was rotated away. It never works again, so nothing retries
-- it; the character has to log in again, and that replaces the row.
alter table esi_tokens add column rejected_at timestamptz;
