<?php

declare(strict_types=1);

namespace App\Http\Controllers\Api;

use App\Actions\MapSolarsystem\DeleteMapSolarsystemAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Actions\MapSolarsystem\UpdateMapSolarsystemAction;
use App\Http\Controllers\Controller;
use App\Http\Requests\StoreMapSolarsystemRequest;
use App\Http\Requests\UpdateMapSolarsystemRequest;
use App\Models\MapSolarsystem;
use Illuminate\Http\JsonResponse;
use Symfony\Component\HttpFoundation\Response;
use Throwable;

final class MapSolarsystemController extends Controller
{
    public function update(UpdateMapSolarsystemRequest $request, MapSolarsystem $mapSolarsystem, UpdateMapSolarsystemAction $action): JsonResponse
    {
        $action->handle($mapSolarsystem, $request->validated());

        return response()->json([
            'message' => 'Solarsystem updated!',
            'data' => $mapSolarsystem->refresh(),
        ], Response::HTTP_CREATED);
    }

    public function store(StoreMapSolarsystemRequest $request, StoreMapSolarsystemAction $action): JsonResponse
    {
        $mapSolarsystem = $action->handle($request->map, $request->validated());

        return response()->json([
            'message' => 'Solarsystem created!',
            'data' => $mapSolarsystem,
        ], Response::HTTP_CREATED);
    }

    /**
     * @throws Throwable
     */
    public function destroy(MapSolarsystem $mapSolarsystem, DeleteMapSolarsystemAction $action): JsonResponse
    {
        $action->handle($mapSolarsystem);

        return response()->json([
            'message' => 'Solarsystem deleted!',
        ], Response::HTTP_NO_CONTENT);
    }
}
