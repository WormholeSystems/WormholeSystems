<?php

declare(strict_types=1);

namespace App\Console\Commands\Characters;

use App\Console\Commands\AppCommand;
use App\Events\Characters\CharacterStatusUpdatedEvent;
use App\Models\CharacterStatus;
use App\Models\Map;
use Illuminate\Console\Command;
use Illuminate\Database\Eloquent\Collection;
use NicolasKion\Esi\Esi;
use Throwable;

use function now;
use function sprintf;

final class GetOnlineCharactersCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:get-online-characters';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Checks the online status of characters and updates their status in the database';

    public function __construct(
        private readonly Esi $esi,
    ) {
        parent::__construct();
    }

    /**
     * Execute the console command.
     */
    public function handle(): void
    {
        $this->markInactiveCharactersAsOffline();

        $characters = $this->getRecentlyActiveCharacterIds();

        $updated_character_ids = $characters->map($this->checkCharacterStatus(...))->filter();

        if ($updated_character_ids->isEmpty()) {
            $this->info('No character status changes detected.');

            return;
        }

        Map::query()
            ->whereHas('mapAccessors', fn ($query) => $query->whereIn('accessible_id', $updated_character_ids))
            ->each(fn ($map) => CharacterStatusUpdatedEvent::dispatch($map->id));
    }

    private function checkCharacterStatus(CharacterStatus $characterStatus): ?int
    {
        try {
            $request = $this->esi->getOnline($characterStatus->character);
        } catch (Throwable $e) {
            $this->error(sprintf('Connection error while checking status for character %d: %s', $characterStatus->character_id, $e->getMessage()), log: true);

            return null;
        }

        if ($request->failed()) {
            $this->error(sprintf('Failed to get online status for character %d', $characterStatus->character_id), log: true);

            return null;
        }

        $this->info(sprintf('Character %d is %s', $characterStatus->character_id, $request->data->online ? 'online' : 'offline'));

        $online_status_changed = $characterStatus->is_online !== $request->data->online;

        $characterStatus->update([
            'is_online' => $request->data->online,
            'last_online_at' => $request->data->online ? now() : $characterStatus->last_online_at,
            'online_last_checked_at' => now(),
        ]);

        if ($online_status_changed) {
            $this->info(sprintf('Character %d status changed to %s', $characterStatus->character_id, $request->data->online ? 'online' : 'offline'));

            return $characterStatus->character_id;
        }

        return null;
    }

    private function markInactiveCharactersAsOffline(): void
    {
        CharacterStatus::query()
            ->isOnline()
            ->wasNotRecentlyActive()
            ->update([
                'is_online' => false,
                'online_last_checked_at' => now(),
            ]);
    }

    /**
     * @return Collection<int, CharacterStatus>
     */
    private function getRecentlyActiveCharacterIds(): Collection
    {
        return CharacterStatus::query()
            ->wasRecentlyActive()
            ->hasRequiredScopes()
            ->get();
    }
}
