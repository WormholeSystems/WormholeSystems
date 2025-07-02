<?php

namespace App\Http\Controllers;

use App\Http\Resources\MapResource;
use App\Http\Resources\SolarsystemResource;
use App\Models\Map;
use App\Models\Solarsystem;
use Illuminate\Http\Request;
use Inertia\Inertia;
use Inertia\Response;

class MapController extends Controller
{
    /**
     * @throws \Throwable
     */
    public function show(Request $request, Map $map): Response
    {
        $map->load('mapSolarsystems', 'mapConnections');

        $search = $request->string('search');

        $solarsystems = Solarsystem::query()->whereLike('name', sprintf('%s%%', $search->value()))
            ->limit(10)->get()->toResourceCollection(SolarsystemResource::class);

        return Inertia::render('Map/ShowMap', [
            'map' => $map->toResource(MapResource::class),
            'solarsystems' => $solarsystems,
            'search' => $search,
        ]);
    }
}
