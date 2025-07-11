<?php

namespace App\Http\Controllers;

use App\Enums\Permission;
use App\Http\Requests\UpdateMapAccessRequest;
use App\Http\Resources\MapResource;
use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;
use Symfony\Component\HttpFoundation\RedirectResponse;
use Throwable;

class MapAccessController extends Controller
{
    public function __construct(
        #[CurrentUser] protected ?User $user,
    ) {}

    /**
     * @throws Throwable
     */
    public function show(Request $request, Map $map): Response
    {
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

        return Inertia::render('maps/ShowMapAccess', [
            'map' => $map->toResource(MapResource::class),
            'entities' => $entities,
            'search' => $search,
        ]);
    }

    public function store(UpdateMapAccessRequest $request, Map $map): RedirectResponse
    {

        Gate::authorize('create', [MapAccess::class, $map]);

        if ($request->map_access) {
            if (! Gate::check('update', [MapAccess::class, $map, $request->map_access])) {
                return back()->notify('Access denied.', 'You do not have permission to change this character access.');
            }
            $permission = $request->enum('permission', Permission::class);

            $map_access = $request->map_access;
            if ($permission) {
                $map_access->permission = $permission;
                $map_access->save();
            } else {
                $map_access->delete();
            }

            return back()->notify('Access updated successfully.', 'The access for the entity has been updated.');
        }

        $map->mapAccessors()->create([
            'accessible_type' => sprintf('App\\Models\\%s', $request->string('entity_type')->ucfirst()),
            'accessible_id' => $request->integer('entity_id'),
            'permission' => $request->enum('permission', Permission::class),
        ]);

        return back()->notify('Access updated successfully.', 'You have successfully granted access to the entity.');
    }
}
