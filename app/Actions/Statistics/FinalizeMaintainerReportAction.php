<?php

declare(strict_types=1);

namespace App\Actions\Statistics;

use App\DTO\MaintainerCharacterStat;
use App\DTO\MaintainerSettings;
use App\Models\Map;
use App\Models\MapMaintainerReport;
use App\Services\Statistics\MaintainerLeaderboard;
use App\Services\Statistics\MaintainerPeriod;
use Carbon\CarbonImmutable;
use InvalidArgumentException;

final readonly class FinalizeMaintainerReportAction
{
    public function __construct(
        private MaintainerLeaderboard $leaderboard,
    ) {}

    public function handle(Map $map, MaintainerPeriod $period): MapMaintainerReport
    {
        if (CarbonImmutable::now('UTC')->lt($period->endsAt())) {
            throw new InvalidArgumentException(
                "Cannot finalize maintainer period \"{$period->toString()}\" before it has ended.",
            );
        }

        $characters = $this->leaderboard->details($map, $period);
        $settings = MaintainerSettings::fromMap($map);

        $payload = [
            'characters' => $characters
                ->map(fn (MaintainerCharacterStat $stat): array => $stat->toArray())
                ->all(),
            'settings' => $settings->toArray(),
        ];

        // Only payload/generated_at are written -- discord_posted_at must never be touched
        // here, or re-finalizing an already-posted period would resurrect its post.
        return MapMaintainerReport::query()->updateOrCreate(
            ['map_id' => $map->id, 'period' => $period->toString()],
            ['payload' => $payload, 'generated_at' => CarbonImmutable::now('UTC')],
        );
    }
}
