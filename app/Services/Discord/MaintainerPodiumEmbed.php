<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\DTO\MaintainerCharacterStat;
use App\DTO\MaintainerEntry;
use App\Models\MapAlert;
use App\Services\Statistics\MaintainerPeriod;
use Illuminate\Support\Collection;

/**
 * Builds the Discord embed for the monthly maintainer podium: the top scanners/maintainers
 * on a map, ranked by the map's configured point weights. Entries arrive already grouped
 * and positioned by MaintainerLeaderboard::aggregate() and already threshold-filtered by
 * the caller (PostMaintainerPodiumCommand) -- this class only formats and truncates them.
 */
final readonly class MaintainerPodiumEmbed
{
    private const int MAX_ENTRIES = 10;

    private const array MEDALS = [1 => '🥇', 2 => '🥈', 3 => '🥉'];

    /**
     * @param  Collection<int, MaintainerEntry>  $entries  Already threshold-filtered, sorted by points descending.
     * @param  int  $totalScorers  Count before the threshold filter -- used to decide whether the "Full list"
     *                             link is needed, since the in-app leaderboard can show more than qualified here.
     * @return array<string, mixed>
     */
    public function build(MapAlert $alert, MaintainerPeriod $period, Collection $entries, int $totalScorers): array
    {
        $shown = $entries->take(self::MAX_ENTRIES);

        return [
            'title' => sprintf('Maintainer Podium — %s', $period->label()),
            'url' => route('maps.leaderboard.show', [$alert->map, 'period' => $period->toString()]),
            'description' => $shown->isEmpty()
                ? 'Nobody reached the minimum points for this month\'s recap.'
                : $shown->map(fn (MaintainerEntry $entry): string => $this->line($entry))->implode("\n"),
            'fields' => $totalScorers > $shown->count() ? [
                [
                    'name' => 'Full list',
                    'value' => sprintf(
                        '[See the full leaderboard](%s)',
                        route('maps.leaderboard.show', [$alert->map, 'period' => $period->toString()]),
                    ),
                ],
            ] : [],
        ];
    }

    private function line(MaintainerEntry $entry): string
    {
        $rank = self::MEDALS[$entry->position] ?? sprintf('#%d', $entry->position);

        return sprintf('%s **%s** — %d pts (%s)', $rank, $entry->display_name, $entry->points, $this->breakdown($entry));
    }

    private function breakdown(MaintainerEntry $entry): string
    {
        $added = array_sum(array_map(fn (MaintainerCharacterStat $stat): int => $stat->nb_added, $entry->characters));
        $edited = array_sum(array_map(fn (MaintainerCharacterStat $stat): int => $stat->nb_edited, $entry->characters));
        $deleted = array_sum(array_map(fn (MaintainerCharacterStat $stat): int => $stat->nb_deleted, $entry->characters));

        return sprintf('+%d / ~%d / -%d', $added, $edited, $deleted);
    }
}
