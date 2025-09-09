<?php

declare(strict_types=1);

namespace App\Collections;

use App\DTO\OrganisationStats;
use Illuminate\Support\Collection;

/**
 * @extends Collection<int, OrganisationStats>
 */
final class OrganisationStatsCollection extends Collection
{
    public function addActivity(int $organisationId, string $type, string $date): self
    {
        if (! $this->has($organisationId)) {
            $this->put($organisationId, OrganisationStats::create($type));
        }

        $this->put($organisationId, $this->get($organisationId)->addKill($date));

        return $this;
    }

    public function filterByMinimumActiveDays(int $minimumDays): self
    {
        return $this->filter(
            fn (OrganisationStats $stats): bool => $stats->getActiveDayCount() >= $minimumDays
        );
    }

    public function topByKillCount(int $limit): self
    {
        return $this->sortByDesc('killCount')->take($limit);
    }

    public function getTotalKills(): int
    {
        return $this->sum('killCount');
    }

    public function getOrganisationIds(): Collection
    {
        return $this->keys();
    }
}
