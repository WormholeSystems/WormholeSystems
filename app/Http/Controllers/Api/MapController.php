<?php

namespace App\Http\Controllers\Api;

use App\Actions\Map\UpdateMapAction;
use App\Http\Controllers\Controller;
use App\Http\Requests\UpdateMapRequest;
use App\Http\Resources\MapResource;
use App\Models\Map;
use App\Scopes\WithVisibleSolarsystems;
use Illuminate\Http\JsonResponse;
use Throwable;

class MapController extends Controller
{
    /**
     * @throws Throwable
     */
    public function index(): JsonResponse
    {
        return response()->json([
            'data' => Map::all()->toResourceCollection(MapResource::class),
        ]);
    }

    /**
     * @throws Throwable
     */
    public function show(Map $map): JsonResponse
    {
        $map = Map::query()
            ->tap(new WithVisibleSolarsystems)
            ->findOrFail($map->id);

        return response()->json([
            'data' => $map->toResource(MapResource::class),
        ]);
    }

    /**
     * @throws Throwable
     */
    public function update(UpdateMapRequest $request, Map $map, UpdateMapAction $action): JsonResponse
    {
        $action->handle($map, $request->validated());

        return response()->json([
            'message' => 'Map updated!',
            'data' => $map->refresh()->toResource(MapResource::class),
        ]);
    }
}
