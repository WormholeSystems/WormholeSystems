<?php

declare(strict_types=1);

namespace App\DTO;

use App\Collections\OrganisationStatsCollection;
use App\Enums\MapSolarsystemStatus;

final readonly class SystemAnalysisResult
{
    public function __construct(
        public MapSolarsystemStatus $status,
        public string $notesContent,
        public OrganisationStatsCollection $topOrganisations,
        public int $totalKills,
    ) {}

    public static function create(
        OrganisationStatsCollection $topOrganisations,
        AnalysisOptions $options
    ): self {
        $totalKills = $topOrganisations->getTotalKills();

        $status = match (true) {
            $totalKills >= $options->hostileThreshold => MapSolarsystemStatus::Hostile,
            $totalKills >= $options->activeThreshold => MapSolarsystemStatus::Active,
            default => MapSolarsystemStatus::Unknown
        };

        $notesContent = self::generateNotesContent($topOrganisations, $options);

        return new self(
            status: $status,
            notesContent: $notesContent,
            topOrganisations: $topOrganisations,
            totalKills: $totalKills,
        );
    }

    private static function generateNotesContent(OrganisationStatsCollection $topOrganisations, AnalysisOptions $options): string
    {
        $start = '<!-- killmails:start -->';
        $end = '<!-- killmails:end -->';

        if ($topOrganisations->isEmpty()) {
            $markdown = 'We could not find any groups that meet the criteria.';
        } else {
            $markdown = $topOrganisations
                ->map(fn (OrganisationStats $stats, int $id): string => EntityDetails::fromId($id, $stats->killCount)->toMarkdown())
                ->implode("\n");
        }

        return sprintf(
            "%s\nTop %d groups that were active for at least %d days within the last %d days:\n\n%s\n\n%s\n",
            $start,
            $options->top,
            $options->daysActive,
            $options->daysAgo,
            $markdown,
            $end
        );
    }
}
