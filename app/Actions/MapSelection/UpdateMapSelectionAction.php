<?php

declare(strict_types=1);

namespace App\Actions\MapSelection;

use App\Models\MapSolarsystem;
use App\Support\Broadcasting\MapBroadcaster;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class UpdateMapSelectionAction
{
    public function __construct(private MapBroadcaster $mapBroadcaster) {}

    /**
     * @throws Throwable
     */
    public function handle(array $data): void
    {
        DB::transaction(function () use ($data): void {

            $collection = collect($data['map_solarsystems'])->keyBy('id');

            $models = MapSolarsystem::query()
                ->whereIn('id', $collection->pluck('id'))
                ->get();

            $models->each(function (MapSolarsystem $mapSolarsystem) use ($collection): void {
                $values = $collection->get($mapSolarsystem->id, []);
                $mapSolarsystem->update([
                    'position_x' => $values['position_x'] ?? $mapSolarsystem->position_x,
                    'position_y' => $values['position_y'] ?? $mapSolarsystem->position_y,
                ]);
            });

            // A selection always belongs to a single map.
            $map_id = $models->first()?->map_id;

            if ($map_id === null) {
                return;
            }

            $this->mapBroadcaster->systemsUpserted($map_id, MapSolarsystem::query()
                ->whereIn('id', $models->pluck('id'))
                ->with('details')
                ->withCount('signatures', 'wormholeSignatures', 'mapConnections', 'uncategorizedSignatures')
                ->get());
        });
    }
}
