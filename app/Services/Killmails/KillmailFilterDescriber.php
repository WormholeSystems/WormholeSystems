<?php

declare(strict_types=1);

namespace App\Services\Killmails;

use App\Enums\KillmailFilterSubject;
use App\Models\Alliance;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\Group;
use App\Models\Type;
use Illuminate\Support\Collection;

/**
 * Renders a webhook's filter rules as human-readable lines, resolving raw EVE ids to
 * names from the database where possible (e.g. a ship group id to "Battleship").
 */
final class KillmailFilterDescriber
{
    /**
     * @param  Collection<int, KillmailFilterRule>  $filters
     * @return string[]
     */
    public function describe(Collection $filters): array
    {
        return $filters->map(fn (KillmailFilterRule $rule): string => $this->describeRule($rule))->all();
    }

    private function describeRule(KillmailFilterRule $rule): string
    {
        return sprintf(
            '%s (%s) — %s: %s',
            $rule->subject->label(),
            mb_strtolower($rule->side->label()),
            $rule->mode->label(),
            implode(', ', $this->resolveNames($rule->subject, $rule->ids)),
        );
    }

    /**
     * @param  int[]  $ids
     * @return string[]
     */
    private function resolveNames(KillmailFilterSubject $subject, array $ids): array
    {
        $names = $this->lookup($subject, $ids);

        return array_map(fn (int $id): string => $names[$id] ?? (string) $id, $ids);
    }

    /**
     * @param  int[]  $ids
     * @return array<int, string>
     */
    private function lookup(KillmailFilterSubject $subject, array $ids): array
    {
        $model = match ($subject) {
            KillmailFilterSubject::ShipType => Type::class,
            KillmailFilterSubject::ShipGroup => Group::class,
            KillmailFilterSubject::Character => Character::class,
            KillmailFilterSubject::Corporation => Corporation::class,
            KillmailFilterSubject::Alliance => Alliance::class,
        };

        return $model::query()->whereIn('id', $ids)->pluck('name', 'id')->all();
    }
}
