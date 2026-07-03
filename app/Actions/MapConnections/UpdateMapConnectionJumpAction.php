<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Models\MapConnectionJump;
use App\Models\Type;

/**
 * Applies a manual correction to a logged jump (tracked or manual) and pushes
 * the refreshed mass summary to the map.
 *
 * @phpstan-type JumpCorrectionData array{direction?: string, ship_type_id?: int|null, mass?: int}
 */
final readonly class UpdateMapConnectionJumpAction
{
    public function __construct(private BroadcastMapConnectionAction $broadcastMapConnection) {}

    /**
     * @param  JumpCorrectionData  $data
     */
    public function handle(MapConnectionJump $jump, array $data): MapConnectionJump
    {
        $connection = $jump->mapConnection()->with('fromMapSolarsystem', 'toMapSolarsystem')->firstOrFail();

        $attributes = [];

        if (array_key_exists('direction', $data)) {
            $outbound = $data['direction'] === 'outbound';
            $attributes['from_solarsystem_id'] = $outbound
                ? $connection->fromMapSolarsystem->solarsystem_id
                : $connection->toMapSolarsystem->solarsystem_id;
            $attributes['to_solarsystem_id'] = $outbound
                ? $connection->toMapSolarsystem->solarsystem_id
                : $connection->fromMapSolarsystem->solarsystem_id;
        }

        if (array_key_exists('ship_type_id', $data)) {
            $attributes['ship_type_id'] = $data['ship_type_id'];
        }

        if (array_key_exists('mass', $data)) {
            $attributes['mass'] = $data['mass'];
        } elseif (isset($data['ship_type_id'])) {
            $attributes['mass'] = (int) round(Type::query()->whereKey($data['ship_type_id'])->value('mass') ?? 0);
        }

        $jump->update($attributes);

        $this->broadcastMapConnection->handle($connection);

        return $jump;
    }
}
