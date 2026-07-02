<?php

declare(strict_types=1);

namespace App\Actions\MapSelection;

use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Support\Broadcasting\MapBroadcaster;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Facades\DB;
use Throwable;

final class DeleteMapSelectionAction
{
    public function __construct(private readonly MapBroadcaster $mapBroadcaster) {}

    /**
     * Remove the selected systems from the map. Placement rows are hard-deleted so their
     * connections and signatures cascade away; the persistent details survive.
     *
     * @param  int[]  $map_solarsystem_ids
     *
     * @throws Throwable
     */
    public function handle(array $map_solarsystem_ids): void
    {
        DB::transaction(function () use ($map_solarsystem_ids): void {
            // A selection always belongs to a single map.
            $map_id = MapSolarsystem::query()->whereIn('id', $map_solarsystem_ids)->value('map_id');

            if ($map_id === null) {
                return;
            }

            $deleted_system_ids = MapSolarsystem::query()->whereIn('id', $map_solarsystem_ids)->pluck('id')->all();
            $deleted_connection_ids = MapConnection::query()
                ->where(fn (Builder $query) => $query
                    ->whereIn('from_map_solarsystem_id', $deleted_system_ids)
                    ->orWhereIn('to_map_solarsystem_id', $deleted_system_ids))
                ->pluck('id')
                ->all();

            MapSolarsystem::query()->whereIn('id', $map_solarsystem_ids)->delete();

            $this->mapBroadcaster->systemsRemoved($map_id, $deleted_system_ids, $deleted_connection_ids);
        });
    }
}
