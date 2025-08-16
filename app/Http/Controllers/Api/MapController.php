<?php

declare(strict_types=1);

namespace App\Http\Controllers\Api;

use App\Actions\Map\UpdateMapAction;
use App\Http\Controllers\Controller;
use App\Http\Requests\UpdateMapRequest;
use App\Http\Resources\MapResource;
use App\Models\Map;
use App\Models\User;
use App\Scopes\WithVisibleSolarsystems;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;
use Throwable;

final class MapController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user
    ) {}

    /**
     * @throws Throwable
     */
    public function index(Request $request): JsonResponse
    {
        Gate::authorize('viewAny', Map::class);

        $search = $request->string('search', '');

        $maps = Map::query()
            ->whereHas('mapAccessors', fn (Builder $builder) => $builder->whereIn('accessible_id', $this->user->getAccessibleIds()))
            ->when($search->isNotEmpty(), fn (Builder $query) => $query->whereLike('name', sprintf('%%%s%%', $search)))
            ->withCount([
                'mapSolarsystems' => fn (Builder $builder) => $builder->whereNotNull('position_x'),
            ])
            ->with('mapUserSetting')
            ->get()
            ->toResourceCollection(MapResource::class);

        return response()->json([
            'data' => $maps,
        ]);
    }

    /**
     * @throws Throwable
     */
    public function show(Map $map): JsonResponse
    {
        Gate::authorize('view', $map);

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
        Gate::authorize('update', $map);

        $action->handle($map, $request->validated());

        return response()->json([
            'message' => 'Map updated successfully.',
            'data' => $map->refresh()->toResource(MapResource::class),
        ]);
    }
}
