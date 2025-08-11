<?php

declare(strict_types=1);

namespace App\Actions\Tracking;

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Enums\MassStatus;
use App\Enums\ShipSize;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use App\Utilities\WormholeConnectionClassifier;
use Illuminate\Container\Attributes\Config;
use Illuminate\Support\Facades\DB;
use Random\RandomException;
use Throwable;

final class StoreTrackingAction
{
    private const int MINIMUM_DISTANCE_Y = 40;

    private const int MINIMUM_DISTANCE_X = 120;

    public function __construct(
        private WormholeConnectionClassifier $connectionClassifier,
        private StoreMapSolarsystemAction $storeMapSolarsystemAction,
        private CreateMapConnectionAction $storeMapConnectionRequest,
        #[Config('map.max_size.x')] private int $max_x,
        #[Config('map.max_size.y')] private int $max_y
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

            [
                'position_x' => $position_x,
                'position_y' => $position_y,
            ] = $this->guessGoodPositionForNewSolarsystem(
                $map_solarsystem,
            );

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

    /**
     * @return array{position_x: int, position_y: int}
     *
     * @throws RandomException
     */
    private function getRandomPositionAroundSolarsystem(
        MapSolarsystem $mapSolarsystem,
    ): array {
        $distance_x = random_int(self::MINIMUM_DISTANCE_X, self::MINIMUM_DISTANCE_X * 2);
        $distance_y = random_int(self::MINIMUM_DISTANCE_Y, self::MINIMUM_DISTANCE_Y * 2);
        $direction_y = random_int(0, 1) !== 0 ? 1 : -1;
        $position_x = max(40, min($this->max_x, $mapSolarsystem->position_x + $distance_x));
        $position_y = max(20, min($this->max_y, $mapSolarsystem->position_y + ($distance_y * $direction_y)));

        return [
            'position_x' => $position_x,
            'position_y' => $position_y,
        ];

    }

    /**
     * @return array{position_x: int, position_y: int}
     *
     * @throws RandomException
     */
    private function guessGoodPositionForNewSolarsystem(
        MapSolarsystem $mapSolarsystem,
    ): array {

        /**
         * We want to get a good new position for the map solarsystem. Wormholes should be grouped
         */
        $latest_created_map_connection = $mapSolarsystem->connections->sortByDesc('created_at')
            ->filter(function (MapConnection $mapConnection) use ($mapSolarsystem): bool {
                if ($mapSolarsystem->alias === null) {
                    return true;
                }
                $other_map_solarsystem = $mapConnection->toMapSolarsystem->is(
                    $mapSolarsystem)
                    ? $mapConnection->fromMapSolarsystem
                    : $mapConnection->toMapSolarsystem;
                if ($other_map_solarsystem->alias === null) {
                    return true;
                }
                if (! is_numeric($other_map_solarsystem->alias)) {
                    return true;
                }

                return $other_map_solarsystem->alias > $mapSolarsystem->alias;
            })
            ->first();

        if ($latest_created_map_connection === null) {
            return $this->getRandomPositionAroundSolarsystem($mapSolarsystem);
        }

        $latest_created_map_solarsystem = $latest_created_map_connection->toMapSolarsystem->is($mapSolarsystem)
            ? $latest_created_map_connection->fromMapSolarsystem
            : $latest_created_map_connection->toMapSolarsystem;

        $latest_position_x = $latest_created_map_solarsystem->position_x;
        $latest_position_y = $latest_created_map_solarsystem->position_y;

        $new_y = $latest_position_y + self::MINIMUM_DISTANCE_Y;
        $new_x = $latest_position_x;

        if ($new_y > $this->max_y || $new_x > $this->max_x) {
            return $this->getRandomPositionAroundSolarsystem($mapSolarsystem);
        }

        return [
            'position_x' => (int) $new_x,
            'position_y' => (int) $new_y,
        ];
    }
}
