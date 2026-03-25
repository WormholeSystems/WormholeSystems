<?php

declare(strict_types=1);

namespace App\DTO;

use App\Collections\OrganisationStatsCollection;
use App\Enums\ThreatLevel;

final readonly class SystemAnalysisResult
{
    public function __construct(
        public ThreatLevel $threatLevel,
        public array $threatData,
        public OrganisationStatsCollection $topOrganisations,
        public int $totalKills,
    ) {}

    public static function create(
        OrganisationStatsCollection $topOrganisations,
        AnalysisOptions $options
    ): self {
        $totalKills = $topOrganisations->getTotalKills();

        $threatLevel = match (true) {
            $totalKills >= $options->hostileThreshold => ThreatLevel::Critical,
            $totalKills >= $options->activeThreshold => ThreatLevel::High,
            default => ThreatLevel::Unknown,
        };

        $threatData = self::generateThreatData($topOrganisations);

        return new self(
            threatLevel: $threatLevel,
            threatData: $threatData,
            topOrganisations: $topOrganisations,
            totalKills: $totalKills,
        );
    }

    /**
     * @return array<int, array{id: int, name: string, type: string, kills: int}>
     */
    private static function generateThreatData(OrganisationStatsCollection $topOrganisations): array
    {
        if ($topOrganisations->isEmpty()) {
            return [];
        }

        return $topOrganisations
            ->toBase()
            ->map(fn (OrganisationStats $stats, int $id): array => EntityDetails::fromId($id, $stats->killCount)->toArray())
            ->values()
            ->all();
    }
}
