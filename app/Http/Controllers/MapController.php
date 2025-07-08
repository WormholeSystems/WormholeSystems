<?php

namespace App\Http\Controllers;

use App\Http\Integrations\zKillboard\zKillboard;
use App\Http\Resources\KillmailResource;
use App\Http\Resources\MapResource;
use App\Http\Resources\MapSolarsystemResource;
use App\Http\Resources\SolarsystemResource;
use App\Models\Killmail;
use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use App\Scopes\WithVisibleSolarsystems;
use Illuminate\Http\Request;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

class MapController extends Controller
{
    /**
     * @throws Throwable
     */
    public function show(Request $request, Map $map, zKillboard $zKillboard): Response
    {
        $map = Map::query()
            ->tap(new WithVisibleSolarsystems)
            ->findOrFail($map->id);

        $search = $request->string('search');

        $solarsystems = Solarsystem::query()->whereLike('name', sprintf('%s%%', $search->value()))
            ->limit(10)->get()->toResourceCollection(SolarsystemResource::class);

        $selected_map_solarsystem_id = $request->integer('map_solarsystem_id');

        $selected_map_solarsystem = $this->getSelectedSolarsystem($selected_map_solarsystem_id);

        $map_killmails = Inertia::defer(
            fn () => $this->getMapKills($map)
        );

        return Inertia::render('Map/ShowMap', [
            'map' => $map->toResource(MapResource::class),
            'solarsystems' => $solarsystems,
            'search' => $search,
            'config' => config('map'),
            'selected_map_solarsystem' => $selected_map_solarsystem?->toResource(MapSolarsystemResource::class),
            'map_killmails' => $map_killmails,
        ]);
    }

    /**
     * @throws Throwable
     */
    public function getSelectedSolarsystem(?int $solarsystem_id): ?MapSolarsystem
    {
        if (! $solarsystem_id) {
            return null;
        }

        return MapSolarsystem::query()
            ->with('signatures')
            ->findOrFail($solarsystem_id);
    }

    /**
     * @throws Throwable
     */
    public function getMapKills(Map $map)
    {
        return Killmail::query()->with('shipType')
            ->whereIn('solarsystem_id', $map->mapSolarsystems->pluck('solarsystem_id'))
            ->orderBy('time', 'desc')
            ->limit(50)
            ->get()
            ->toResourceCollection(KillmailResource::class);
    }
}
