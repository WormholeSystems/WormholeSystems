<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Actions\EnsureOrganisationExistsAction;
use App\Console\Commands\AppCommand;
use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\Killmail;
use Illuminate\Support\Collection;

use function Laravel\Prompts\progress;

final class BackfillKillmailOrganisationsCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:backfill-killmail-organisations';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Backfill missing corporations and alliances from killmail victims';

    /**
     * Execute the console command.
     */
    public function handle(EnsureOrganisationExistsAction $ensureOrganisationExistsAction): int
    {
        $missingCorporationIds = $this->getMissingCorporationIds();
        $missingAllianceIds = $this->getMissingAllianceIds();

        $this->info(sprintf('Found %d missing corporations and %d missing alliances.', $missingCorporationIds->count(), $missingAllianceIds->count()));

        if ($missingCorporationIds->isEmpty() && $missingAllianceIds->isEmpty()) {
            $this->info('No missing organisations to backfill.');

            return self::SUCCESS;
        }

        if ($missingCorporationIds->isNotEmpty()) {
            progress(
                label: 'Backfilling corporations...',
                steps: $missingCorporationIds,
                callback: fn (int $id) => $ensureOrganisationExistsAction->ensureCorporationExists($id),
            );
        }

        if ($missingAllianceIds->isNotEmpty()) {
            progress(
                label: 'Backfilling alliances...',
                steps: $missingAllianceIds,
                callback: fn (int $id) => $ensureOrganisationExistsAction->ensureAllianceExists($id),
            );
        }

        $this->info('Backfill complete.');

        return self::SUCCESS;
    }

    /**
     * @return Collection<int, int<1, max>>
     */
    private function getMissingCorporationIds(): Collection
    {
        $victimCorpIds = Killmail::query()
            ->selectRaw("DISTINCT JSON_UNQUOTE(JSON_EXTRACT(`data`, '$.victim.corporation_id')) as corp_id")
            ->pluck('corp_id')
            ->map(fn ($id): int => (int) $id)
            ->filter(fn (int $id): bool => $id > 0)
            ->values();

        $existingCorpIds = Corporation::query()
            ->whereIn('id', $victimCorpIds)
            ->pluck('id');

        return $victimCorpIds->diff($existingCorpIds)->values();
    }

    /**
     * @return Collection<int, int<1, max>>
     */
    private function getMissingAllianceIds(): Collection
    {
        $victimAllianceIds = Killmail::query()
            ->selectRaw("DISTINCT JSON_UNQUOTE(JSON_EXTRACT(`data`, '$.victim.alliance_id')) as alliance_id")
            ->pluck('alliance_id')
            ->map(fn ($id): int => (int) $id)
            ->filter(fn (int $id): bool => $id > 0)
            ->values();

        $existingAllianceIds = Alliance::query()
            ->whereIn('id', $victimAllianceIds)
            ->pluck('id');

        return $victimAllianceIds->diff($existingAllianceIds)->values();
    }
}
