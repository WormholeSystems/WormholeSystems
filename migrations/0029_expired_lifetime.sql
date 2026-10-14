-- The in-game "Reliable Lifetime: Expired, closure imminent": past critical, closing any
-- moment. Last in the enum, so worst-wins sync and the ageing's escalate-only compare
-- treat it as the worst there is. Only a pilot sets it; the clock stops at critical.
alter type time_status add value 'expired';
