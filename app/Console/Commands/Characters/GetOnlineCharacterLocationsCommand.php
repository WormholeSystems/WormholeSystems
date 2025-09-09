<?php

declare(strict_types=1);

namespace App\Console\Commands\Characters;

use App\Actions\ShipHistories\UpdateShipHistoryAction;
use App\Console\Commands\AppCommand;
use App\Events\Characters\CharacterStatusUpdatedEvent;
use App\Models\Character;
use App\Models\CharacterStatus;
use App\Models\Map;
use Illuminate\Console\Command;
use Illuminate\Http\Client\ConnectionException;
use NicolasKion\Esi\DTO\Location;
use NicolasKion\Esi\DTO\Ship;
use NicolasKion\Esi\Esi;
use Throwable;

use function assert;
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

    public function __construct(
        private readonly Esi $esi,
        private readonly UpdateShipHistoryAction $action,
    ) {
        parent::__construct();
    }

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

        $updated_character_ids = $characters->map($this->checkCharacterStatus(...))->filter();

        if ($updated_character_ids->isEmpty()) {
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

    /**
     * @throws Throwable
     * @throws ConnectionException
     */
    private function checkCharacterStatus(CharacterStatus $characterStatus): ?int
    {
        $location_request = $this->esi->getLocation($characterStatus->character);

        if ($location_request->failed()) {
            $this->error(sprintf('Failed to get online status for character %d', $characterStatus->character_id));

            return null;
        }

        $ship_request = $this->esi->getShip($characterStatus->character);

        if ($ship_request->failed()) {
            $this->error(sprintf('Failed to get ship details for character %d', $characterStatus->character_id));

            return null;
        }

        $location = $location_request->data;
        $ship = $ship_request->data;

        assert($location instanceof Location);
        assert($ship instanceof Ship);

        $characterStatus->update([
            'solarsystem_id' => $location->solar_system_id,
            'station_id' => $location->station_id,
            'structure_id' => $location->structure_id,
            'ship_name' => $ship->ship_name,
            'ship_type_id' => $ship->ship_type_id,
            'ship_item_id' => $ship->ship_item_id,
        ]);

        $this->action->handle(
            $characterStatus->character_id,
            $ship->ship_item_id,
            $ship->ship_type_id,
            $ship->ship_name
        );

        if ($characterStatus->wasChanged()) {
            $this->info(sprintf('Updated character %d: Ship "%s" (Type ID: %d), Location: %d',
                $characterStatus->character_id,
                $characterStatus->ship_name,
                $characterStatus->ship_type_id,
                $characterStatus->solarsystem_id,
            ));

            return $characterStatus->character_id;
        }

        $this->info(sprintf('No changes for character %d', $characterStatus->character_id));

        return null;
    }
}
