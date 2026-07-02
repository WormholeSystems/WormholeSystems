<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Enums\ConnectionType;
use App\Models\MapConnection;
use App\Models\MapConnectionJump;
use Illuminate\Database\Eloquent\Builder;

/**
 * Attaches recent pending jump rows (recorded before the connection existed)
 * to a freshly created connection matching their map and solar system pair.
 */
final class ClaimPendingConnectionJumpsAction
{
    /**
     * Pendings older than this are not claimed: the longer the gap between
     * the observed jump and the connection creation, the higher the risk of
     * attributing a jump to an unrelated hole between the same systems.
     */
    private const int CLAIM_WINDOW_SECONDS = 120;

    public function handle(MapConnection $mapConnection): void
    {
        /* A freshly created connection reports a null type (the database-default
         * is not hydrated yet), so only an explicit stargate type skips the claim.
         */
        if ($mapConnection->type === ConnectionType::Stargate) {
            return;
        }

        $from_solarsystem_id = $mapConnection->fromMapSolarsystem->solarsystem_id;
        $to_solarsystem_id = $mapConnection->toMapSolarsystem->solarsystem_id;

        MapConnectionJump::query()
            ->whereNull('map_connection_id')
            ->where('map_id', $mapConnection->map_id)
            ->where('created_at', '>=', now()->subSeconds(self::CLAIM_WINDOW_SECONDS))
            ->where(fn (Builder $query) => $query
                ->where(fn (Builder $query) => $query
                    ->where('from_solarsystem_id', $from_solarsystem_id)
                    ->where('to_solarsystem_id', $to_solarsystem_id)
                )
                ->orWhere(fn (Builder $query) => $query
                    ->where('from_solarsystem_id', $to_solarsystem_id)
                    ->where('to_solarsystem_id', $from_solarsystem_id)
                )
            )
            ->update(['map_connection_id' => $mapConnection->id]);
    }
}
