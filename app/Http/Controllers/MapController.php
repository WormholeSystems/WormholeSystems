<?php

namespace App\Http\Controllers;

use App\Http\Resources\CharacterResource;
use App\Http\Resources\KillmailResource;
use App\Http\Resources\MapResource;
use App\Http\Resources\MapSolarsystemResource;
use App\Http\Resources\SolarsystemResource;
use App\Models\Character;
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
    public function show(Request $request, Map $map): Response
    {
        $map = Map::query()
            ->tap(new WithVisibleSolarsystems)
            ->findOrFail($map->id);

        $search = $request->string('search');

        $solarsystems = Solarsystem::query()->whereLike('name', sprintf('%s%%', $search->value()))
            ->limit(10)->get()->toResourceCollection(SolarsystemResource::class);

        $selected_map_solarsystem_id = $request->integer('map_solarsystem_id');

        $selected_map_solarsystem = fn () => $this->getSelectedSolarsystem($selected_map_solarsystem_id)?->toResource(MapSolarsystemResource::class);

        $map_killmails = Inertia::defer(
            fn () => $this->getMapKills($map)
        );

        $map_characters = fn () => $this->getMapCharacters($map);

        return Inertia::render('Maps/ShowMap', [
            'map' => $map->toResource(MapResource::class),
            'solarsystems' => $solarsystems,
            'search' => $search,
            'config' => config('map'),
            'selected_map_solarsystem' => $selected_map_solarsystem,
            'map_killmails' => $map_killmails,
            'map_characters' => $map_characters,
        ]);
    }

    /**
     * @throws Throwable
     */
    public function index(): Response
    {
        return Inertia::render('Maps/ShowAllMaps', [
            'maps' => Map::query()->get()->toResourceCollection(MapResource::class),
        ]);
    }

    /**
     * @throws Throwable
     */
    public function getSelectedSolarsystem(?int $solarsystem_id): ?MapSolarsystem
    {
        if ($solarsystem_id === null || $solarsystem_id === 0) {
            return null;
        }

        return MapSolarsystem::query()
            ->with('signatures')
            ->findOrFail($solarsystem_id);
    }

    /**
     * @throws Throwable
     */
    public function getMapKills(Map $map): \Illuminate\Http\Resources\Json\ResourceCollection
    {
        return Killmail::query()->with('shipType')
            ->whereIn('solarsystem_id', $map->mapSolarsystems->pluck('solarsystem_id'))
            ->orderBy('time', 'desc')
            ->limit(50)
            ->get()
            ->toResourceCollection(KillmailResource::class);
    }

    /**
     * @throws Throwable
     */
    public function getMapCharacters(Map $map): \Illuminate\Http\Resources\Json\ResourceCollection
    {
        return Character::query()
            ->with('characterStatus')
            ->whereRelation('characterStatus', 'is_online', true)
            ->get()
            ->toResourceCollection(CharacterResource::class);
    }
}
