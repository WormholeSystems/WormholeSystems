<?php

declare(strict_types=1);

namespace App\Console\Commands\Characters;

use App\Console\Commands\AppCommand;
use App\Jobs\Characters\DispatchCharacterStatusEvents;
use App\Jobs\Characters\UpdateCharacterLocation;
use App\Models\CharacterStatus;
use Illuminate\Bus\Batch;
use Illuminate\Support\Facades\Bus;
use Throwable;

use function sprintf;

final class GetOnlineCharacterLocationsCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:get-online-character-locations';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Checks the location of online characters and updates their status in the database';

    /**
     * Execute the console command.
     *
     * @throws Throwable
     */
    public function handle(): void
    {
        $characters = CharacterStatus::query()
            ->isOnline()
            ->hasRequiredScopes()
            ->get();

        CharacterStatus::query()
            ->doesntHaveRequiredScopes()
            ->update([
                'is_online' => false,
            ]);

        if ($characters->isEmpty()) {
            $this->info('No online characters to process.');

            return;
        }

        $this->info(sprintf('Dispatching jobs for %d characters...', $characters->count()));

        $startTime = now()->toIso8601String();
        $jobs = $characters->map(fn (CharacterStatus $characterStatus): UpdateCharacterLocation => new UpdateCharacterLocation($characterStatus->id));

        $batch = Bus::batch($jobs->all())
            ->name('Update Character Locations')
            ->finally(function (Batch $batch) use ($startTime): void {
                // Dispatch a job to handle event dispatching after all character updates are complete
                DispatchCharacterStatusEvents::dispatch($startTime);
            })
            ->dispatch();

        $this->info(sprintf('Batch dispatched with ID: %s', $batch->id));
        $this->info('Characters will be processed in parallel.');
    }
}
