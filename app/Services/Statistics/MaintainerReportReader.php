<?php

declare(strict_types=1);

namespace App\Services\Statistics;

use App\Actions\Statistics\FinalizeMaintainerReportAction;
use App\DTO\MaintainerCharacterStat;
use App\DTO\MaintainerEntry;
use App\Models\Map;
use App\Models\MapMaintainerReport;
use Illuminate\Database\QueryException;
use Illuminate\Support\Collection;

use function app;
use function str_contains;

/**
 * The single read path for the stats API and the settings page. The current month is
 * always live-computed; a finished month is finalized on demand (if not already) and
 * read back from its frozen payload -- so weights and character->user mutations made
 * after a period ended can never change what was already shown.
 */
final readonly class MaintainerReportReader
{
    public function __construct(
        private MaintainerLeaderboard $leaderboard,
    ) {}

    /**
     * @return Collection<int, MaintainerCharacterStat>
     */
    public function details(Map $map, MaintainerPeriod $period): Collection
    {
        if ($period->isCurrent()) {
            return $this->leaderboard->details($map, $period);
        }

        return $this->statsFromReport($this->reportFor($map, $period));
    }

    /**
     * @return Collection<int, MaintainerEntry>
     */
    public function aggregated(Map $map, MaintainerPeriod $period): Collection
    {
        if ($period->isCurrent()) {
            return $this->leaderboard->aggregated($map, $period);
        }

        return $this->leaderboard->aggregate($this->statsFromReport($this->reportFor($map, $period)));
    }

    private function reportFor(Map $map, MaintainerPeriod $period): MapMaintainerReport
    {
        $report = MapMaintainerReport::query()
            ->where('map_id', $map->id)
            ->where('period', $period->toString())
            ->first();

        if ($report !== null) {
            return $report;
        }

        try {
            return app(FinalizeMaintainerReportAction::class)->handle($map, $period);
        } catch (QueryException $exception) {
            if (! $this->isDuplicateKeyViolation($exception)) {
                throw $exception;
            }

            // Two concurrent first-reads of the same just-ended month raced updateOrCreate;
            // the other request won, so re-read instead of surfacing a 500 to either viewer.
            return MapMaintainerReport::query()
                ->where('map_id', $map->id)
                ->where('period', $period->toString())
                ->firstOrFail();
        }
    }

    private function isDuplicateKeyViolation(QueryException $exception): bool
    {
        return (string) $exception->getCode() === '23000' || str_contains($exception->getMessage(), 'Duplicate entry');
    }

    /**
     * @return Collection<int, MaintainerCharacterStat>
     */
    private function statsFromReport(MapMaintainerReport $report): Collection
    {
        return collect($report->payload['characters'])
            ->map(fn (array $row): MaintainerCharacterStat => MaintainerCharacterStat::fromArray($row));
    }
}
