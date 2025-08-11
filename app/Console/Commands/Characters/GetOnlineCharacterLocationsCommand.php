<?php

declare(strict_types=1);

namespace App\Console\Commands\Characters;

use App\Actions\ShipHistories\UpdateShipHistoryAction;
use App\Events\Characters\CharacterStatusUpdatedEvent;
use App\Models\Character;
use App\Models\CharacterStatus;
use App\Models\Map;
use App\Scopes\CharacterDoesntHaveRequiredScopes;
use App\Scopes\CharacterHasRequiredScopes;
use Illuminate\Console\Command;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Facades\Log;
use NicolasKion\Esi\DTO\Location;
use NicolasKion\Esi\DTO\Ship;
use NicolasKion\Esi\Enums\EsiScope;
use NicolasKion\Esi\Esi;
use Throwable;

final class GetOnlineCharacterLocationsCommand extends Command
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
     * @throws Throwable
     */
    public function handle(Esi $esi, UpdateShipHistoryAction $action): void
    {
        $characters = CharacterStatus::query()
            ->where('is_online', true)
            ->whereHas('character', fn (Builder $query) => $query
                ->tap(new CharacterHasRequiredScopes([
                    EsiScope::ReadOnlineStatus,
                    EsiScope::ReadLocations,
                    EsiScope::ReadShip,
                ])))
            ->get();

        CharacterStatus::query()
            ->whereHas('character', fn (Builder $query) => $query
                ->tap(new CharacterDoesntHaveRequiredScopes([
                    EsiScope::ReadOnlineStatus,
                    EsiScope::ReadLocations,
                    EsiScope::ReadShip,
                ])))
            ->update([
                'is_online' => false,
            ]);

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

            /** @var Location $location */
            $location = $location_request->data;

            /** @var Ship $ship */
            $ship = $ship_request->data;

            $character->update([
                'solarsystem_id' => $location->solar_system_id,
                'station_id' => $location->station_id,
                'structure_id' => $location->structure_id,
                'ship_name' => $ship->ship_name,
                'ship_type_id' => $ship->ship_type_id,
                'ship_item_id' => $ship->ship_item_id,
            ]);

            $action->handle(
                $character->character_id,
                $ship->ship_item_id,
                $ship->ship_type_id,
                $ship->ship_name
            );

            Log::info('Updated character location', [
                'character_id' => $character->character_id,
                'solar_system_id' => $location->solar_system_id,
                'station_id' => $location->station_id,
                'structure_id' => $location->structure_id,
                'ship_name' => $ship->ship_name,
                'ship_type_id' => $ship->ship_type_id,
                'ship_item_id' => $ship->ship_item_id,
            ]);

            if ($character->wasChanged()) {
                $updated_character_ids[] = $character->character_id;
                $this->info(sprintf('Updated character %d location to system %d', $character->character_id, $location->solar_system_id));
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

        $accessible_ids = $characters->map(fn (Character $character): array => [
            $character->id,
            $character->corporation_id,
            $character->alliance_id,
        ])->flatten()->whereNotNull()->unique()->values();

        Map::query()
            ->whereHas('mapAccessors', fn ($query) => $query->whereIn('accessible_id', $accessible_ids))
            ->each(fn (Map $map) => CharacterStatusUpdatedEvent::dispatch($map->id));

    }
}
