<?php

declare(strict_types=1);

namespace App\Actions\MapSolarsystem;

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Jobs\Webhooks\EvaluateMapWebhooksJob;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Support\Broadcasting\MapBroadcaster;
use Illuminate\Database\Eloquent\Builder;

final readonly class StoreMapSolarsystemAction
{
    public function __construct(
        private CreateMapConnectionAction $createMapConnection,
        private MapBroadcaster $mapBroadcaster,
    ) {}

    public function handle(Map $map, array $data): MapSolarsystem
    {
        // Persistent intel is kept (or created with defaults) so it survives a system being
        // removed and re-added; only the placement is (re)created.
        $details = $map->mapSolarsystemDetails()->firstOrCreate([
            'solarsystem_id' => $data['solarsystem_id'],
        ]);

        $map_solarsystem = $map->mapSolarsystems()->firstOrNew([
            'solarsystem_id' => $data['solarsystem_id'],
        ]);
        $map_solarsystem->map_solarsystem_details_id = $details->id;

        // Only a newly placed system takes the requested position; re-adding one already on
        // the map (e.g. to connect to it) must leave it where the user put it.
        if (! $map_solarsystem->exists) {
            $map_solarsystem->position_x = $data['position_x'];
            $map_solarsystem->position_y = $data['position_y'];
        }
        $map_solarsystem->save();

        EvaluateMapWebhooksJob::dispatch($map->id, $map_solarsystem->solarsystem_id)->afterCommit();

        $this->connectToOrigin($map, $map_solarsystem, $data['connect_to_map_solarsystem_id'] ?? null);

        $this->mapBroadcaster->systemsUpserted($map->id, MapSolarsystem::query()
            ->whereKey($map_solarsystem->id)
            ->with('details')
            ->withCount('signatures', 'wormholeSignatures', 'mapConnections', 'uncategorizedSignatures')
            ->get());

        return $map_solarsystem;
    }

    /**
     * Link the freshly placed system back to the system it was added from (when the user
     * used "Add connection"), skipping self-links and any connection that already exists.
     */
    private function connectToOrigin(Map $map, MapSolarsystem $target, ?int $origin_id): void
    {
        if ($origin_id === null || $origin_id === $target->id) {
            return;
        }

        // Each direction must AND its from/to pair, so the two pairs are wrapped in their
        // own closures — array-form where()/orWhere() would join the keys with the call's
        // boolean and turn the second pair into an OR, matching any link touching either node.
        $already_connected = MapConnection::query()
            ->where('map_id', $map->id)
            ->where(function (Builder $query) use ($origin_id, $target): void {
                $query
                    ->where(fn (Builder $pair) => $pair
                        ->where('from_map_solarsystem_id', $origin_id)
                        ->where('to_map_solarsystem_id', $target->id))
                    ->orWhere(fn (Builder $pair) => $pair
                        ->where('from_map_solarsystem_id', $target->id)
                        ->where('to_map_solarsystem_id', $origin_id));
            })
            ->exists();

        if ($already_connected) {
            return;
        }

        $this->createMapConnection->handle([
            'from_map_solarsystem_id' => $origin_id,
            'to_map_solarsystem_id' => $target->id,
        ]);
    }
}
