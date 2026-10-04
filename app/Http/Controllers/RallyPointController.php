<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Models\Map;
use App\Support\Broadcasting\MapBroadcaster;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Gate;

final class RallyPointController extends Controller
{
    /**
     * Adds a rally point. Once the map holds the maximum number of rally points,
     * the oldest one makes room for the new one.
     */
    public function store(Request $request, Map $map, MapBroadcaster $mapBroadcaster): RedirectResponse
    {
        Gate::authorize('update', $map);

        $validated = $request->validate([
            'solarsystem_id' => ['required', 'integer', 'exists:solarsystems,id'],
        ]);

        $solarsystem_id = (int) $validated['solarsystem_id'];

        $this->updateRallySolarsystemIds($map, fn (array $ids): array => array_slice(
            [...array_diff($ids, [$solarsystem_id]), $solarsystem_id],
            -config()->integer('map.max_rally_points'),
        ));

        $mapBroadcaster->metadataUpdated($map);

        return back();
    }

    public function destroy(Request $request, Map $map, MapBroadcaster $mapBroadcaster): RedirectResponse
    {
        Gate::authorize('update', $map);

        $validated = $request->validate([
            'solarsystem_id' => ['required', 'integer'],
        ]);

        $solarsystem_id = (int) $validated['solarsystem_id'];

        $this->updateRallySolarsystemIds($map, fn (array $ids): array => array_diff($ids, [$solarsystem_id]));

        $mapBroadcaster->metadataUpdated($map);

        return back();
    }

    /**
     * Applies the change on a locked row so concurrent edits never drop each other's rally points.
     *
     * @param  callable(list<int>): array<int, int>  $callback
     */
    private function updateRallySolarsystemIds(Map $map, callable $callback): void
    {
        DB::transaction(function () use ($map, $callback): void {
            $current = Map::query()->lockForUpdate()->findOrFail($map->id)->rally_solarsystem_ids;

            $map->update(['rally_solarsystem_ids' => array_values($callback($current))]);
        });
    }
}
