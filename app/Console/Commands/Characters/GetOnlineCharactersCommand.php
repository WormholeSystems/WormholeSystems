<?php

namespace App\Console\Commands\Characters;

use App\Events\Characters\CharacterStatusUpdatedEvent;
use App\Models\CharacterStatus;
use App\Models\Map;
use Illuminate\Console\Command;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\Client\ConnectionException;
use NicolasKion\Esi\Esi;

class GetOnlineCharactersCommand extends Command
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

    /**
     * Execute the console command.
     *
     * @throws ConnectionException
     */
    public function handle(Esi $esi): void
    {
        $characters = CharacterStatus::query()
            ->whereHas('character.user', fn (Builder $query) => $query->where('last_active_at', '>=', now()->subMinutes(10)))
            ->get();

        $updated_character_ids = [];

        foreach ($characters as $character) {
            $request = $esi->getOnline($character->character);

            if ($request->failed()) {
                $this->error(sprintf('Failed to get online status for character %d', $character->character_id));

                continue;
            }

            $this->info(sprintf('Character %d is %s', $character->character_id, $request->data->online ? 'online' : 'offline'));

            $online_status_changed = $character->is_online !== $request->data->online;

            $character->update([
                'is_online' => $request->data->online,
                'last_online_at' => $request->data->online ? now() : $character->last_online_at,
                'online_last_checked_at' => now(),
            ]);

            if ($online_status_changed) {
                $updated_character_ids[] = $character->character_id;
                $this->info(sprintf('Character %d status changed to %s', $character->character_id, $request->data->online ? 'online' : 'offline'));
            }
        }

        if ($updated_character_ids === []) {
            $this->info('No character status changes detected.');

            return;
        }

        Map::query()
            ->whereHas('mapAccessors', fn ($query) => $query->whereIn('accessible_id', $updated_character_ids))
            ->each(fn ($map) => CharacterStatusUpdatedEvent::dispatch($map->id));
    }
}
