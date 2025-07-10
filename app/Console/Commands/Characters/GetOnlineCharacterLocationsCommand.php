<?php

namespace App\Console\Commands\Characters;

use App\Events\Characters\CharacterStatusUpdatedEvent;
use App\Models\Character;
use App\Models\CharacterStatus;
use App\Models\Map;
use Illuminate\Console\Command;
use Illuminate\Http\Client\ConnectionException;
use NicolasKion\Esi\Esi;

class GetOnlineCharacterLocationsCommand extends Command
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
     * @throws ConnectionException
     */
    public function handle(Esi $esi): void
    {
        $characters = CharacterStatus::query()
            ->where('is_online', true)
            ->get();

        $updated_character_ids = [];

        foreach ($characters as $character) {
            $location_request = $esi->getLocation($character->character);

            if ($location_request->failed()) {
                $this->error(sprintf('Failed to get online status for character %d', $character->character_id));

                continue;
            }

            $ship_request = $esi->getShip($character->character);

            if ($ship_request->failed()) {
                $this->error(sprintf('Failed to get ship details for character %d', $character->character_id));

                continue;
            }

            $character->update([
                'solarsystem_id' => $location_request->data->solar_system_id,
                'station_id' => $location_request->data->station_id,
                'structure_id' => $location_request->data->structure_id,
                'ship_name' => $ship_request->data->ship_name,
                'ship_type_id' => $ship_request->data->ship_type_id,
            ]);

            if ($character->wasChanged()) {
                $updated_character_ids[] = $character->character_id;
                $this->info(sprintf('Updated character %d location to system %d', $character->character_id, $location_request->data->solar_system_id));
            } else {
                $this->info(sprintf('No changes for character %d', $character->character_id));
            }
        }

        if ($updated_character_ids === []) {
            $this->info('No characters were updated.');

            return;
        }

        $characters = Character::query()
            ->whereIn('id', $updated_character_ids)
            ->get();

        $accessible_ids = $characters->map(fn (Character $character) => [
            $character->id,
            $character->corporation_id,
            $character->alliance_id,
        ])->flatten()->whereNotNull()->unique()->values();

        Map::query()
            ->whereHas('mapAccessors', fn ($query) => $query->whereIn('accessible_id', $accessible_ids))
            ->each(fn (Map $map) => CharacterStatusUpdatedEvent::dispatch($map->id));

    }
}
