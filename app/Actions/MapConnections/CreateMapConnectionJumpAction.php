<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Models\MapConnection;
use App\Models\MapConnectionJump;
use App\Models\Type;

/**
 * Creates a manually logged jump on a connection, e.g. for untracked pilots
 * whose passage would otherwise be missing from the mass estimate.
 *
 * @phpstan-type ManualJumpData array{map_connection_id: int, direction: string, ship_type_id?: int|null, mass?: int|null}
 */
final readonly class CreateMapConnectionJumpAction
{
    public function __construct(private BroadcastMapConnectionAction $broadcastMapConnection) {}

    /**
     * @param  ManualJumpData  $data
     */
    public function handle(array $data): MapConnectionJump
    {
        $connection = MapConnection::query()
            ->with('fromMapSolarsystem', 'toMapSolarsystem')
            ->findOrFail($data['map_connection_id']);

        $outbound = $data['direction'] === 'outbound';
        $ship_type_id = $data['ship_type_id'] ?? null;

        $jump = MapConnectionJump::query()->create([
            'map_id' => $connection->map_id,
            'map_connection_id' => $connection->id,
            'character_id' => null,
            'from_solarsystem_id' => $outbound
                ? $connection->fromMapSolarsystem->solarsystem_id
                : $connection->toMapSolarsystem->solarsystem_id,
            'to_solarsystem_id' => $outbound
                ? $connection->toMapSolarsystem->solarsystem_id
                : $connection->fromMapSolarsystem->solarsystem_id,
            'ship_type_id' => $ship_type_id,
            'mass' => $data['mass'] ?? $this->shipTypeMass($ship_type_id),
            'is_manual' => true,
        ]);

        $this->broadcastMapConnection->handle($connection);

        return $jump;
    }

    private function shipTypeMass(?int $ship_type_id): int
    {
        return (int) round(Type::query()->whereKey($ship_type_id)->value('mass') ?? 0);
    }
}
