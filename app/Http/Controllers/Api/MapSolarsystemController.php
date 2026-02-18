<?php

declare(strict_types=1);

namespace App\Http\Controllers\Api;

use App\Actions\MapSolarsystem\DeleteMapSolarsystemAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Actions\MapSolarsystem\UpdateMapSolarsystemAction;
use App\Enums\Permission;
use App\Http\Controllers\Controller;
use App\Http\Requests\StoreMapSolarsystemRequest;
use App\Http\Requests\UpdateMapSolarsystemRequest;
use App\Http\Resources\SelectedMapSolarsystemResource;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Gate;
use Symfony\Component\HttpFoundation\Response;
use Throwable;

final class MapSolarsystemController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user
    ) {}

    /**
     * @throws Throwable
     */
    public function show(MapSolarsystem $mapSolarsystem): JsonResponse
    {
        Gate::authorize('view', $mapSolarsystem);

        $mapSolarsystem->load('signatures');

        $isViewer = $mapSolarsystem->map->getUserPermission($this->user) === Permission::Viewer;
        $mapSolarsystem->hideNotes($isViewer);

        return response()->json([
            'data' => $mapSolarsystem->toResource(SelectedMapSolarsystemResource::class),
        ]);
    }

    public function update(UpdateMapSolarsystemRequest $request, MapSolarsystem $mapSolarsystem, UpdateMapSolarsystemAction $action): JsonResponse
    {
        Gate::authorize('update', $mapSolarsystem);

        $action->handle($mapSolarsystem, $request->validated());

        return response()->json([
            'message' => 'Solarsystem updated successfully.',
        ], status: Response::HTTP_OK);
    }

    public function store(StoreMapSolarsystemRequest $request, StoreMapSolarsystemAction $action): JsonResponse
    {
        Gate::authorize('create', [MapSolarsystem::class, $request->map]);

        $action->handle($request->map, $request->validated());

        return response()->json([
            'message' => 'Solarsystem created successfully.',
        ], status: Response::HTTP_CREATED);
    }

    /**
     * @throws Throwable
     */
    public function destroy(MapSolarsystem $mapSolarsystem, DeleteMapSolarsystemAction $action): JsonResponse
    {
        Gate::authorize('delete', $mapSolarsystem);

        $action->handle($mapSolarsystem);

        return response()->json([
            'message' => 'Solarsystem deleted successfully.',
        ], status: Response::HTTP_OK);
    }
}
