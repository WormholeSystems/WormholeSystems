<?php

declare(strict_types=1);

namespace App\Services\Statistics;

use App\DTO\MaintainerCharacterStat;
use App\DTO\MaintainerEntry;
use App\DTO\MaintainerSettings;
use App\Models\Map;
use App\Models\SignatureActivity;
use App\Models\User;
use Illuminate\Support\Collection;

final readonly class MaintainerLeaderboard
{
    /**
     * One row per character, unfiltered by the map's minimum-points threshold.
     *
     * @return Collection<int, MaintainerCharacterStat>
     */
    public function details(Map $map, MaintainerPeriod $period): Collection
    {
        return $this->rows($map, $period, MaintainerSettings::fromMap($map));
    }

    /**
     * Grouped by user, zero-point entries dropped, sorted by points descending. The map's
     * minimum-points threshold is a Discord-recap-only cutoff applied by the caller, not here.
     *
     * @return Collection<int, MaintainerEntry>
     */
    public function aggregated(Map $map, MaintainerPeriod $period): Collection
    {
        return $this->aggregate($this->details($map, $period));
    }

    /**
     * Pure grouping/position derivation over an already-computed stat collection -- used both
     * for the live path and the stored path, so both are provably identical. The only DB access
     * here is the single preferred-character lookup below.
     *
     * @param  Collection<int, MaintainerCharacterStat>  $stats
     * @return Collection<int, MaintainerEntry>
     */
    public function aggregate(Collection $stats): Collection
    {
        $userIds = $stats->pluck('user_id')->filter()->unique()->values();

        /** @var array<int, int|null> $preferredCharacterIdsByUser */
        $preferredCharacterIdsByUser = User::query()
            ->whereIn('id', $userIds)
            ->pluck('preferred_character_id', 'id')
            ->all();

        $grouped = $stats->groupBy(
            fn (MaintainerCharacterStat $stat): string => $stat->user_id !== null
                ? 'user:'.$stat->user_id
                : 'character:'.$stat->character_id,
        );

        $unpositioned = $grouped
            ->map(function (Collection $characters) use ($preferredCharacterIdsByUser): array {
                $userId = $characters->first()->user_id;
                $points = (int) $characters->sum(fn (MaintainerCharacterStat $stat): int => $stat->points);

                return [
                    'points' => $points,
                    'user_id' => $userId,
                    'display_name' => $this->displayName($characters, $userId, $preferredCharacterIdsByUser),
                    'characters' => $characters->values()->all(),
                ];
            })
            ->filter(fn (array $entry): bool => $entry['points'] >= 1)
            ->values()
            ->all();

        usort($unpositioned, function (array $a, array $b): int {
            if ($a['points'] !== $b['points']) {
                return $b['points'] <=> $a['points'];
            }

            $aUserId = $a['user_id'] ?? PHP_INT_MAX;
            $bUserId = $b['user_id'] ?? PHP_INT_MAX;

            if ($aUserId !== $bUserId) {
                return $aUserId <=> $bUserId;
            }

            /** @var MaintainerCharacterStat $aCharacter */
            $aCharacter = $a['characters'][0];
            /** @var MaintainerCharacterStat $bCharacter */
            $bCharacter = $b['characters'][0];

            return $aCharacter->character_id <=> $bCharacter->character_id;
        });

        return collect($unpositioned)->values()->map(
            fn (array $entry, int $index): MaintainerEntry => new MaintainerEntry(
                position: $index + 1,
                points: $entry['points'],
                user_id: $entry['user_id'],
                display_name: $entry['display_name'],
                characters: $entry['characters'],
            ),
        );
    }

    /**
     * @param  Collection<int, MaintainerCharacterStat>  $characters
     * @param  array<int, int|null>  $preferredCharacterIdsByUser
     */
    private function displayName(Collection $characters, ?int $userId, array $preferredCharacterIdsByUser): string
    {
        if ($userId !== null) {
            $preferredCharacterId = $preferredCharacterIdsByUser[$userId] ?? null;

            if ($preferredCharacterId !== null) {
                $preferred = $characters->firstWhere('character_id', $preferredCharacterId);

                if ($preferred !== null) {
                    return $preferred->character_name;
                }
            }
        }

        $top = null;

        foreach ($characters as $character) {
            if ($top === null
                || $character->points > $top->points
                || ($character->points === $top->points && $character->character_id < $top->character_id)) {
                $top = $character;
            }
        }

        return $top->character_name;
    }

    /**
     * @return Collection<int, MaintainerCharacterStat>
     */
    private function rows(Map $map, MaintainerPeriod $period, MaintainerSettings $settings): Collection
    {
        $rows = SignatureActivity::query()
            ->toBase()
            ->join('characters', 'characters.id', '=', 'signature_activities.character_id')
            ->where('signature_activities.map_id', $map->id)
            // Two explicit clauses, not whereBetween: endsAt() is exclusive and whereBetween
            // is inclusive on both ends, which would pull in the 1st of the next month.
            ->where('signature_activities.activity_date', '>=', $period->startsAt()->toDateString())
            ->where('signature_activities.activity_date', '<', $period->endsAt()->toDateString())
            ->groupBy('signature_activities.character_id', 'characters.name', 'characters.user_id')
            ->selectRaw(<<<'SQL'
                signature_activities.character_id,
                characters.name as character_name,
                characters.user_id,
                SUM(action = 'created') as nb_added,
                SUM(action = 'updated') as nb_edited,
                SUM(action = 'deleted') as nb_deleted
            SQL)
            ->get();

        return $rows->map(function (object $row) use ($settings): MaintainerCharacterStat {
            $nbAdded = (int) $row->nb_added;
            $nbEdited = (int) $row->nb_edited;
            $nbDeleted = (int) $row->nb_deleted;

            return new MaintainerCharacterStat(
                character_id: (int) $row->character_id,
                // characters.name is nullable -- a contributor whose ESI name hasn't
                // resolved yet would otherwise hand a null to a non-nullable string property.
                character_name: $row->character_name ?? 'Unknown character',
                user_id: $row->user_id !== null ? (int) $row->user_id : null,
                nb_added: $nbAdded,
                nb_edited: $nbEdited,
                nb_deleted: $nbDeleted,
                points: $nbAdded * $settings->points_created
                    + $nbEdited * $settings->points_updated
                    + $nbDeleted * $settings->points_deleted,
            );
        })->values();
    }
}
