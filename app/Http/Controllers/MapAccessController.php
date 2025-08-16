<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapAccess\CreateMapAccessAction;
use App\Actions\MapAccess\DeleteMapAccessAction;
use App\Actions\MapAccess\UpdateMapAccessAction;
use App\Enums\Permission;
use App\Http\Requests\UpdateMapAccessRequest;
use App\Http\Resources\MapResource;
use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\Map;
use App\Models\MapAccess;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;
use Symfony\Component\HttpFoundation\RedirectResponse;
use Throwable;

final class MapAccessController extends Controller
{
    public function __construct() {}

    /**
     * @throws Throwable
     */
    public function show(Request $request, Map $map): Response
    {
        Gate::authorize('update', $map); // Only users with write access can manage access

        $search = $request->string('search');
        $entities = DB::query()->fromSub(DB::table('characters')
            ->selectRaw('id, name, "character" as type, (
                SELECT permission
                FROM map_access
                WHERE accessible_type = "App\\\\Models\\\\Character"
                AND accessible_id = characters.id
                AND map_id = ?
            ) as permission', [$map->id])->unionAll(
                Corporation::query()->selectRaw('id, name, "corporation" as type, (
                    SELECT permission
                    FROM map_access
                    WHERE accessible_type = "App\\\\Models\\\\Corporation"
                    AND accessible_id = corporations.id
                    AND map_id = ?
                ) as permission', [$map->id])
            )->unionAll(
                Alliance::query()->selectRaw('id, name, "alliance" as type, (
                    SELECT permission
                    FROM map_access
                    WHERE accessible_type = "App\\\\Models\\\\Alliance"
                    AND accessible_id = alliances.id
                    AND map_id = ?
                ) as permission', [$map->id])
            ), 'entities')
            ->whereLike('name', sprintf('%s%%', $search))
            ->orderBy('name')
            ->take(30)
            ->select(['id', 'name', 'type', 'permission'])
            ->get();

        return Inertia::render('maps/settings/ShowAccess', [
            'map' => $map->toResource(MapResource::class),
            'entities' => $entities,
            'search' => $search,
            'has_write_access' => true, // If we reach here, user has write access (due to gate check)
        ]);
    }

    /**
     * @throws Throwable
     */
    public function store(
        UpdateMapAccessRequest $request,
        Map $map,
        CreateMapAccessAction $createMapAccessAction,
        UpdateMapAccessAction $updateMapAccessAction,
        DeleteMapAccessAction $deleteMapAccessAction
    ): RedirectResponse {
        if ($this->isUpdatingExistingAccess($request)) {
            if (! $this->canUpdateExistingAccess($request, $map)) {
                return back()->notify('Access denied.', 'You do not have permission to change this character access.');
            }

            if (($permission = $request->permission) instanceof Permission) {
                $updateMapAccessAction->handle($request->map_access, permission: $permission);

                return back()->notify('Access updated successfully.', sprintf('The access for %s has been updated.', $request->map_access->accessible->name));
            }

            $name = $request->map_access->accessible->name;

            $deleteMapAccessAction->handle($request->map_access);

            return back()->notify('Access removed successfully.', sprintf('The access for %s has been removed.', $name));
        }

        $createMapAccessAction->handle($map, $request->accessor, permission: $request->permission);

        return back()->notify('Access updated successfully.', 'You have successfully granted access to the entity.');
    }

    private function isUpdatingExistingAccess(UpdateMapAccessRequest $request): bool
    {
        return $request->map_access instanceof MapAccess;
    }

    private function canUpdateExistingAccess(UpdateMapAccessRequest $request, Map $map): bool
    {
        return Gate::check('update', [MapAccess::class, $map, $request->map_access]);
    }
}
