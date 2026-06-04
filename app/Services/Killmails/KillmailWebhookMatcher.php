<?php

declare(strict_types=1);

namespace App\Services\Killmails;

use App\Enums\KillmailFilterMatch;
use App\Enums\KillmailFilterMode;
use App\Enums\KillmailFilterSide;
use App\Enums\KillmailFilterSubject;
use Illuminate\Support\Collection;

/**
 * Decides whether a killmail satisfies a webhook's filter rules. Pure logic — the
 * caller resolves ship-type → group ids, then indexes the killmail's candidate ids once
 * via buildPools() and queries those pools against each webhook's filters. No database
 * access, so it stays trivially unit-testable.
 *
 * Include rules combine according to $match (any = OR, all = AND); exclude rules always
 * veto a match. Within a single rule the configured ids are OR'd. An empty rule set
 * always matches.
 */
final class KillmailWebhookMatcher
{
    /**
     * @param  array{victim: array<string, int[]>, attacker: array<string, int[]>}  $pools  From {@see buildPools()}.
     * @param  Collection<int, KillmailFilterRule>  $filters
     */
    public function matches(array $pools, Collection $filters, KillmailFilterMatch $match = KillmailFilterMatch::Any): bool
    {
        if ($filters->isEmpty()) {
            return true;
        }

        $includes = [];
        foreach ($filters as $rule) {
            $ruleMatched = array_intersect($rule->ids, $this->poolForSide($pools, $rule)) !== [];

            if ($rule->mode === KillmailFilterMode::Exclude) {
                if ($ruleMatched) {
                    return false;
                }

                continue;
            }

            $includes[] = $ruleMatched;
        }

        if ($includes === []) {
            return true;
        }

        return $match === KillmailFilterMatch::All
            ? ! in_array(false, $includes, true)
            : in_array(true, $includes, true);
    }

    /**
     * The include rules the killmail satisfies — used to show *why* an alert fired. Each
     * returned rule is narrowed to only the ids that actually matched, so a rule like
     * "ship group in [Battleship, Cruiser]" reports just "Cruiser" on a cruiser loss.
     * Exclude rules are omitted: a fired alert by definition matched none of them.
     *
     * @param  array{victim: array<string, int[]>, attacker: array<string, int[]>}  $pools
     * @param  Collection<int, KillmailFilterRule>  $filters
     * @return Collection<int, KillmailFilterRule>
     */
    public function matchingRules(array $pools, Collection $filters): Collection
    {
        return $filters
            ->filter(fn (KillmailFilterRule $rule): bool => $rule->mode === KillmailFilterMode::Include)
            ->map(function (KillmailFilterRule $rule) use ($pools): ?KillmailFilterRule {
                $matchedIds = array_values(array_intersect($rule->ids, $this->poolForSide($pools, $rule)));

                return $matchedIds === []
                    ? null
                    : new KillmailFilterRule($rule->subject, $rule->side, $rule->mode, $matchedIds);
            })
            ->filter()
            ->values();
    }

    /**
     * Index a killmail's candidate ids per side and subject, once, for repeated matching
     * against many webhooks' filters.
     *
     * @param  array<string, mixed>  $killmailData  Decoded ESI killmail (fully associative).
     * @param  array<int, int>  $typeGroupMap  ship_type_id => group_id
     * @return array{victim: array<string, int[]>, attacker: array<string, int[]>}
     */
    public function buildPools(array $killmailData, array $typeGroupMap = []): array
    {
        $victim = is_array($killmailData['victim'] ?? null) ? $killmailData['victim'] : [];
        $attackers = is_array($killmailData['attackers'] ?? null) ? $killmailData['attackers'] : [];

        return [
            KillmailFilterSide::Victim->value => $this->idsForParticipants([$victim], $typeGroupMap),
            KillmailFilterSide::Attacker->value => $this->idsForParticipants($attackers, $typeGroupMap),
        ];
    }

    /**
     * @param  array<int, mixed>  $participants
     * @param  array<int, int>  $typeGroupMap
     * @return array<string, int[]>
     */
    private function idsForParticipants(array $participants, array $typeGroupMap): array
    {
        $pool = [
            KillmailFilterSubject::Character->value => [],
            KillmailFilterSubject::Corporation->value => [],
            KillmailFilterSubject::Alliance->value => [],
            KillmailFilterSubject::ShipType->value => [],
            KillmailFilterSubject::ShipGroup->value => [],
        ];

        foreach ($participants as $participant) {
            if (! is_array($participant)) {
                continue;
            }

            $this->collect($pool[KillmailFilterSubject::Character->value], $participant['character_id'] ?? null);
            $this->collect($pool[KillmailFilterSubject::Corporation->value], $participant['corporation_id'] ?? null);
            $this->collect($pool[KillmailFilterSubject::Alliance->value], $participant['alliance_id'] ?? null);

            $shipTypeId = isset($participant['ship_type_id']) ? (int) $participant['ship_type_id'] : null;
            $this->collect($pool[KillmailFilterSubject::ShipType->value], $shipTypeId);

            if ($shipTypeId !== null && isset($typeGroupMap[$shipTypeId])) {
                $this->collect($pool[KillmailFilterSubject::ShipGroup->value], $typeGroupMap[$shipTypeId]);
            }
        }

        return array_map(array_values(...), array_map(array_unique(...), $pool));
    }

    /**
     * @param  int[]  $pool
     */
    private function collect(array &$pool, mixed $value): void
    {
        if (in_array($value, [null, '', 0], true)) {
            return;
        }

        $pool[] = (int) $value;
    }

    /**
     * @param  array{victim: array<string, int[]>, attacker: array<string, int[]>}  $pools
     * @return int[]
     */
    private function poolForSide(array $pools, KillmailFilterRule $rule): array
    {
        $subject = $rule->subject->value;

        return match ($rule->side) {
            KillmailFilterSide::Victim => $pools[KillmailFilterSide::Victim->value][$subject],
            KillmailFilterSide::Attacker => $pools[KillmailFilterSide::Attacker->value][$subject],
            KillmailFilterSide::Either => array_values(array_unique(array_merge(
                $pools[KillmailFilterSide::Victim->value][$subject],
                $pools[KillmailFilterSide::Attacker->value][$subject],
            ))),
        };
    }
}
