<?php

declare(strict_types=1);

namespace App\Actions\Tracking;

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapConnections\UpdateMapConnectionAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Actions\Signatures\UpdateSignatureAction;
use App\Data\SignatureData;
use App\Data\TrackingData;
use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Enums\ShipSize;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\Signature;
use App\Models\Solarsystem;
use App\Traits\PositionsMapSolarsystems;
use App\Utilities\WormholeConnectionClassifier;
use Illuminate\Container\Attributes\Config;
use Illuminate\Support\Facades\DB;
use NicolasKion\SDE\Models\SolarsystemConnection;
use Random\RandomException;
use Throwable;

final readonly class StoreTrackingAction
{
    use PositionsMapSolarsystems;

    private const int MINIMUM_DISTANCE_Y = 40;

    private const int MINIMUM_DISTANCE_X = 120;

    private const int MAXIMUM_TRIES = 100;

    public function __construct(
        private WormholeConnectionClassifier $connectionClassifier,
        private StoreMapSolarsystemAction $storeMapSolarsystemAction,
        private CreateMapConnectionAction $storeMapConnectionRequest,
        private UpdateSignatureAction $updateSignatureAction,
        private UpdateMapConnectionAction $updateMapConnectionAction,
        #[Config('map.max_size.x')]
        private int $max_x,
        #[Config('map.max_size.y')]
        private int $max_y,
        #[Config('map.grid_size')]
        private int $grid_size,
    ) {}

    /**
     * @throws Throwable
     */
    public function handle(TrackingData $data): void
    {
        DB::transaction(function () use ($data): void {
            $origin = MapSolarsystem::query()
                ->lockForUpdate()
                ->findOrFail($data->from_map_solarsystem_id);
            $to_solarsystem = Solarsystem::query()
                ->lockForUpdate()
                ->findOrFail($data->to_solarsystem_id);

            if ($this->isSolarsystemOnMap($origin->map, $to_solarsystem)) {
                /* If the target system is already on the map,
                * we do not create a new connection, as it should
                * already exist. We just update the signature if provided.
                */
                $this->updateExistingConnection($origin, $to_solarsystem, $data);

                return;
            }

            if (
                $this->isKSpaceToKSpaceConnection($origin->solarsystem, $to_solarsystem) &&
                $this->systemsAreConnectedPerStargates($origin->solarsystem, $to_solarsystem)
            ) {
                return;
            }

            $ship_size = $this->connectionClassifier->getSize($origin->solarsystem, $to_solarsystem);

            [
                'position_x' => $position_x,
                'position_y' => $position_y,
            ] = $this->guessGoodPositionForNewSolarsystem(
                $origin,
            );

            $new_map_solarsystem = $this->storeMapSolarsystemAction->handle(
                $origin->map,
                [
                    'solarsystem_id' => $to_solarsystem->id,
                    'position_x' => $position_x,
                    'position_y' => $position_y,
                ]
            );

            $signature = Signature::query()->find($data->signature_id);

            $mass_status = $this->getMassStatusForSignature($signature);
            $lifetime_status = $this->getLifetimeStatusForSignature($signature);

            $connection = $this->storeMapConnectionRequest->handle(
                [
                    'from_map_solarsystem_id' => $data->from_map_solarsystem_id,
                    'to_map_solarsystem_id' => $new_map_solarsystem->id,
                    'wormhole_id' => null,
                    'mass_status' => $mass_status,
                    'ship_size' => $ship_size ?? ShipSize::Large,
                    'lifetime' => $lifetime_status,
                ]
            );

            // Link the signature to the connection if provided
            if ($data->signature_id) {
                Signature::query()->where('id', $data->signature_id)
                    ->update([
                        'map_connection_id' => $connection->id,
                    ]);
            }

        }, 10);
    }

    private function isSolarsystemOnMap(Map $map, Solarsystem $solarsystem): bool
    {
        return $map->mapSolarsystems()
            ->isSolarsystem($solarsystem)
            ->isOnMap()
            ->exists();
    }

    private function isKSpaceToKSpaceConnection(Solarsystem $from, Solarsystem $to): bool
    {
        return $from->type === 'eve' && $to->type === 'eve';
    }

    /**
     * @return array{position_x: int, position_y: int}
     *
     * @throws RandomException
     */
    private function getRandomPositionAroundSolarsystem(
        MapSolarsystem $mapSolarsystem,
    ): array {
        return $this->getRandomPositionAroundSystem(
            $mapSolarsystem,
            self::MINIMUM_DISTANCE_X,
            self::MINIMUM_DISTANCE_Y
        );
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
        $latest_created_map_connection = $this->getLatestChildConnection($mapSolarsystem);

        if (! $latest_created_map_connection instanceof MapConnection) {
            return $this->getRandomPositionAroundSolarsystem($mapSolarsystem);
        }

        $latest_created_map_solarsystem = $this->getConnectionTarget($latest_created_map_connection, $mapSolarsystem);

        return $this->getNextFreePosition(
            $mapSolarsystem->map,
            $latest_created_map_solarsystem,
            self::MINIMUM_DISTANCE_Y,
            self::MINIMUM_DISTANCE_X,
            self::MAXIMUM_TRIES
        );

    }

    /**
     * Get the latest created connection that originated from this solarsystem.
     * For example if A -> B and A -> C exist, and B was created before C,
     * we want to return the connection to C.
     */
    private function getLatestChildConnection(MapSolarsystem $mapSolarsystem): ?MapConnection
    {
        return $mapSolarsystem->mapConnections->sortByDesc('created_at')
            ->filter(fn (MapConnection $mapConnection): bool => $this->excludeConnectionsToParent($mapConnection, $mapSolarsystem))
            ->first();
    }

    /**
     * We want to exclude connections that point back to the parent system.
     * For example if we have A -> B and B -> C, and we are looking for
     * connections originating from B, we want to exclude the connection
     * that points back to A (A -> B).
     */
    private function excludeConnectionsToParent(MapConnection $mapConnection, MapSolarsystem $mapSolarsystem): bool
    {
        // If the solarsystem has no alias, we can not infer
        // if the connection is relevant, so we keep it as a candidate
        if ($mapSolarsystem->alias === null) {
            return true;
        }

        $other_map_solarsystem = $this->getConnectionTarget($mapConnection, $mapSolarsystem);

        if ($other_map_solarsystem->alias === null) {
            return true;
        }

        return $other_map_solarsystem->alias > $mapSolarsystem->alias;
    }

    private function getConnectionTarget(MapConnection $mapConnection, MapSolarsystem $origin): MapSolarsystem
    {
        return $mapConnection->toMapSolarsystem->is($origin)
            ? $mapConnection->fromMapSolarsystem
            : $mapConnection->toMapSolarsystem;
    }

    private function systemsAreConnectedPerStargates(Solarsystem $from, Solarsystem $to): bool
    {
        return SolarsystemConnection::query()
            ->where('from_solarsystem_id', $from->id)
            ->where('to_solarsystem_id', $to->id)
            ->exists();
    }

    /**
     * @throws Throwable
     */
    private function updateExistingConnection(
        MapSolarsystem $origin,
        Solarsystem $to_solarsystem,
        TrackingData $data,
    ): void {

        $connection = MapConnection::query()->connectsSolarsystemsInMap($origin->map_id, $origin->solarsystem_id, $to_solarsystem->id)
            ->first();

        if (! $connection instanceof MapConnection) {
            return;
        }

        $signature = Signature::query()->find($data->signature_id);

        if (! $signature instanceof Signature) {
            return;
        }

        $this->updateSignatureAction->handle($signature, SignatureData::from([
            'map_connection_id' => $connection->id,
        ]));
    }

    private function getMassStatusForSignature(?Signature $signature): MassStatus
    {
        if (! $signature instanceof Signature) {
            return MassStatus::Fresh;
        }

        return $signature->mass_status;
    }

    private function getLifetimeStatusForSignature(?Signature $signature): LifetimeStatus
    {
        if (! $signature instanceof Signature) {
            return LifetimeStatus::Healthy;
        }

        return $signature->lifetime;
    }
}
