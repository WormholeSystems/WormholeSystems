<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Enums\ConnectionType;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapConnectionJump;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use App\Models\Type;
use App\Utilities\StargatePairDetector;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Collection;

/**
 * Records a tracked character's jump between two solar systems against the
 * wormhole connection linking them, on every map that tracks the character.
 *
 * When the connection does not exist yet (the client creates it moments after
 * the location poll observes the jump), a pending row is stored instead and
 * claimed by ClaimPendingConnectionJumpsAction at connection creation.
 */
final readonly class RecordMapConnectionJumpAction
{
    public function __construct(
        private BroadcastMapConnectionAction $broadcastMapConnection,
        private StargatePairDetector $stargatePairDetector,
    ) {}

    public function handle(
        int $character_id,
        int $from_solarsystem_id,
        int $to_solarsystem_id,
        ?int $ship_type_id,
        ?string $ship_name,
    ): void {
        $from_solarsystem = Solarsystem::query()->find($from_solarsystem_id);
        $to_solarsystem = Solarsystem::query()->find($to_solarsystem_id);

        if ($from_solarsystem === null || $to_solarsystem === null) {
            return;
        }

        /* A gate pair means the character travelled through a stargate, not a
         * wormhole — even a mapped wormhole connection between the same pair
         * would make the attribution ambiguous, so we record nothing.
         */
        if ($this->stargatePairDetector->isStargatePair($from_solarsystem, $to_solarsystem)) {
            return;
        }

        $maps = $this->getMapsTrackingCharacter($character_id);

        if ($maps->isEmpty()) {
            return;
        }

        $mass = (int) round(Type::query()->whereKey($ship_type_id)->value('mass') ?? 0);

        foreach ($maps as $map) {
            $this->recordJumpOnMap($map, $character_id, $from_solarsystem_id, $to_solarsystem_id, $ship_type_id, $ship_name, $mass);
        }
    }

    private function recordJumpOnMap(
        Map $map,
        int $character_id,
        int $from_solarsystem_id,
        int $to_solarsystem_id,
        ?int $ship_type_id,
        ?string $ship_name,
        int $mass,
    ): void {
        $connection = $this->findConnection($map->id, $from_solarsystem_id, $to_solarsystem_id);

        if ($connection instanceof MapConnection && $connection->type !== ConnectionType::Wormhole) {
            return;
        }

        if (! $connection instanceof MapConnection && ! $this->originIsOnMap($map->id, $from_solarsystem_id)) {
            return;
        }

        $jump = MapConnectionJump::query()->create([
            'map_id' => $map->id,
            'map_connection_id' => $connection?->id,
            'character_id' => $character_id,
            'from_solarsystem_id' => $from_solarsystem_id,
            'to_solarsystem_id' => $to_solarsystem_id,
            'ship_type_id' => $ship_type_id,
            'ship_name' => $ship_name,
            'mass' => $mass,
        ]);

        /* Pending row: the connection may have been created between our first
         * lookup and the insert (the claim at creation time would then have
         * missed this row), so check once more and self-claim.
         */
        if (! $connection instanceof MapConnection) {
            $connection = $this->findConnection($map->id, $from_solarsystem_id, $to_solarsystem_id);

            if (! $connection instanceof MapConnection || $connection->type !== ConnectionType::Wormhole) {
                return;
            }

            $jump->update(['map_connection_id' => $connection->id]);
        }

        $this->broadcastMapConnection->handle($connection);
    }

    /**
     * @return Collection<int, Map>
     */
    private function getMapsTrackingCharacter(int $character_id): Collection
    {
        return Map::query()
            ->whereHas('mapUserSettings', fn (Builder $query) => $query
                ->where('tracking_allowed', true)
                ->where('is_tracking', true)
                ->whereHas('user.characters', fn (Builder $query) => $query->where('id', $character_id))
            )
            ->get();
    }

    private function findConnection(int $map_id, int $from_solarsystem_id, int $to_solarsystem_id): ?MapConnection
    {
        return MapConnection::query()
            ->connectsSolarsystemsInMap($map_id, $from_solarsystem_id, $to_solarsystem_id)
            ->first();
    }

    private function originIsOnMap(int $map_id, int $solarsystem_id): bool
    {
        return MapSolarsystem::query()
            ->where('map_id', $map_id)
            ->where('solarsystem_id', $solarsystem_id)
            ->whereNotNull('position_x')
            ->exists();
    }
}
