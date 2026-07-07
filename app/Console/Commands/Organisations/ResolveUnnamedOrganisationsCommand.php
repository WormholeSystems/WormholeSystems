<?php

declare(strict_types=1);

namespace App\Console\Commands\Organisations;

use App\Actions\EnsureOrganisationExistsAction;
use App\Console\Commands\AppCommand;
use App\Models\Alliance;
use App\Models\Corporation;

use function Laravel\Prompts\progress;

/** Resolves names for organisations that entered as bare id rows; 404s are marked unresolvable. */
final class ResolveUnnamedOrganisationsCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:resolve-unnamed-organisations {--limit=500 : Maximum organisations to resolve per kind}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Resolve names for unnamed corporations and alliances, marking ones ESI no longer knows';

    /**
     * Execute the console command.
     */
    public function handle(EnsureOrganisationExistsAction $ensureOrganisationExistsAction): int
    {
        $limit = (int) $this->option('limit');

        $corporation_ids = Corporation::query()
            ->whereNull('name')
            ->whereNull('unresolvable_at')
            ->limit($limit)
            ->pluck('id');

        $alliance_ids = Alliance::query()
            ->whereNull('name')
            ->whereNull('unresolvable_at')
            ->limit($limit)
            ->pluck('id');

        $this->info(sprintf('Found %d unnamed corporations and %d unnamed alliances.', $corporation_ids->count(), $alliance_ids->count()));

        if ($corporation_ids->isNotEmpty()) {
            progress(
                label: 'Resolving corporations...',
                steps: $corporation_ids,
                callback: function (int $corporation_id) use ($ensureOrganisationExistsAction): void {
                    $ensureOrganisationExistsAction->refreshCorporation($corporation_id);
                },
            );
        }

        if ($alliance_ids->isNotEmpty()) {
            progress(
                label: 'Resolving alliances...',
                steps: $alliance_ids,
                callback: function (int $alliance_id) use ($ensureOrganisationExistsAction): void {
                    $ensureOrganisationExistsAction->refreshAlliance($alliance_id);
                },
            );
        }

        return self::SUCCESS;
    }
}
