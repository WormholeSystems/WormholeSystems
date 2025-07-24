<?php

namespace App\Actions\Tracking;

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Enums\MassStatus;
use App\Enums\ShipSize;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use App\Utilities\WormholeConnectionClassifier;
use Illuminate\Container\Attributes\Config;
use Illuminate\Support\Facades\DB;
use Throwable;

class StoreTrackingAction
{
    public function __construct(
        protected WormholeConnectionClassifier $connectionClassifier,
        protected StoreMapSolarsystemAction $storeMapSolarsystemAction,
        protected CreateMapConnectionAction $storeMapConnectionRequest,
        #[Config('map.max_size.x')] protected int $max_x,
        #[Config('map.max_size.y')] protected int $max_y
    ) {}

    /**
     * @throws Throwable
     */
    public function handle(array $data): void
    {
        DB::transaction(function () use ($data): void {
            $from_map_solarsystem_id = $data['from_map_solarsystem_id'];
            $to_solarsystem_id = $data['to_solarsystem_id'];

            $map_solarsystem = MapSolarsystem::query()
                ->lockForUpdate()
                ->findOrFail($from_map_solarsystem_id);
            $to_solarsystem = Solarsystem::query()
                ->lockForUpdate()
                ->findOrFail($to_solarsystem_id);

            $solarsystem_already_on_map = $map_solarsystem->map->mapSolarsystems()
                ->where('solarsystem_id', $to_solarsystem->id)
                ->whereNotNull('position_x')
                ->lockForUpdate()
                ->exists();

            if ($solarsystem_already_on_map) {
                return;
            }

            if ($map_solarsystem->solarsystem->type === 'eve' && $to_solarsystem->type === 'eve') {
                // If both systems are known space, we do not create a connection.
                return;
            }

            $ship_size = $this->connectionClassifier->getSize($map_solarsystem->solarsystem, $to_solarsystem);

            $minimum_distance_y = 40;
            $minimum_distance_x = 120;
            $distance_x = random_int($minimum_distance_x, $minimum_distance_x * 2);
            $distance_y = random_int($minimum_distance_y, $minimum_distance_y * 2);
            $direction_y = random_int(0, 1) !== 0 ? 1 : -1;

            $position_x = max(40, min($this->max_x, $map_solarsystem->position_x + $distance_x));
            $position_y = max(20, min($this->max_y, $map_solarsystem->position_y + ($distance_y * $direction_y)));

            $new_map_solarsystem = $this->storeMapSolarsystemAction->handle(
                $map_solarsystem->map,
                [
                    'solarsystem_id' => $to_solarsystem->id,
                    'position_x' => $position_x,
                    'position_y' => $position_y,
                ]
            );

            $this->storeMapConnectionRequest->handle(
                [
                    'from_map_solarsystem_id' => $from_map_solarsystem_id,
                    'to_map_solarsystem_id' => $new_map_solarsystem->id,
                    'wormhole_id' => $data['wormhole_id'] ?? null,
                    'mass_status' => $data['mass_status'] ?? MassStatus::Fresh,
                    'ship_size' => $ship_size ?? ShipSize::Large,
                    'is_eol' => $data['is_eol'] ?? false,
                ]
            );

        }, 10);
    }
}
