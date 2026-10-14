-- The killmails card can follow the selected system instead of a half of the chain. Which
-- system that is lives in the page, not here: the card sends it along with each request.
alter type killmail_scope add value 'selected_system';
