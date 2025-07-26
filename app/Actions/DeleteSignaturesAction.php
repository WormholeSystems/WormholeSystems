<?php

declare(strict_types=1);

namespace App\Actions;

use App\Events\MapConnections\MapConnectionsDeletedEvent;
use App\Models\MapSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteSignaturesAction
{
    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(MapSolarsystem $mapSolarsystem, array $signature_ids): void
    {
        DB::transaction(function () use ($mapSolarsystem, $signature_ids): void {
            $mapSolarsystem->signatures()
                ->whereIn('signatures.id', $signature_ids)
                ->delete();

            broadcast(new MapConnectionsDeletedEvent(
                map_id: $mapSolarsystem->map_id,
            ))->toOthers();
        });
    }
}
