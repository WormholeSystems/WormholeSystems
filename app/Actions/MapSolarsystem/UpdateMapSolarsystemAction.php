<?php

declare(strict_types=1);

namespace App\Actions\MapSolarsystem;

use App\Events\MapSolarsystems\MapSolarsystemsUpdatedEvent;
use App\Models\MapSolarsystem;
use Illuminate\Support\Arr;

final class UpdateMapSolarsystemAction
{
    /**
     * @param  array<string, mixed>  $data
     */
    public function handle(MapSolarsystem $mapSolarsystem, array $data): MapSolarsystem
    {
        $placement_data = Arr::only($data, ['alias', 'position_x', 'position_y', 'pinned']);
        $details_data = Arr::only($data, ['occupier_alias', 'status', 'notes']);

        if ($placement_data !== []) {
            $mapSolarsystem->update($placement_data);
        }

        if ($details_data !== []) {
            // Update the loaded model (not a mass query) so the change is audited.
            $mapSolarsystem->loadMissing('details');
            $mapSolarsystem->details->update($details_data);
        }

        broadcast(new MapSolarsystemsUpdatedEvent($mapSolarsystem->map_id))
            ->toOthers();

        return $mapSolarsystem;
    }
}
