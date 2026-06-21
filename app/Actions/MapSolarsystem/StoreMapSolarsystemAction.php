<?php

declare(strict_types=1);

namespace App\Actions\MapSolarsystem;

use App\Events\MapSolarsystems\MapSolarsystemCreatedEvent;
use App\Jobs\Webhooks\EvaluateMapWebhooksJob;
use App\Models\Map;
use App\Models\MapSolarsystem;

final class StoreMapSolarsystemAction
{
    public function handle(Map $map, array $data): MapSolarsystem
    {
        // Persistent intel is kept (or created with defaults) so it survives a system being
        // removed and re-added; only the placement is (re)created.
        $details = $map->mapSolarsystemDetails()->firstOrCreate([
            'solarsystem_id' => $data['solarsystem_id'],
        ]);

        $map_solarsystem = $map->mapSolarsystems()->updateOrCreate([
            'solarsystem_id' => $data['solarsystem_id'],
        ], [
            'map_solarsystem_details_id' => $details->id,
            'position_x' => $data['position_x'],
            'position_y' => $data['position_y'],
        ]);

        broadcast(new MapSolarsystemCreatedEvent($map->id))
            ->toOthers();

        EvaluateMapWebhooksJob::dispatch($map->id, $map_solarsystem->solarsystem_id)->afterCommit();

        return $map_solarsystem;
    }
}
