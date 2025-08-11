<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapRouteSolarsystems\CreateMapRouteSolarsystemAction;
use App\Actions\MapRouteSolarsystems\DeleteMapRouteSolarsystemAction;
use App\Actions\MapRouteSolarsystems\UpdateMapRouteSolarsystemAction;
use App\Http\Requests\StoreMapRouteSolarsystemRequest;
use App\Http\Requests\UpdateMapRouteSolarsystemRequest;
use App\Models\MapRouteSolarsystem;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Throwable;

final class MapRouteSolarsystemController extends Controller
{
    /**
     * @throws Throwable
     */
    public function store(StoreMapRouteSolarsystemRequest $request, CreateMapRouteSolarsystemAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back()->notify('Route solarsystem created!', message: 'Your route solarsystem has been created successfully.');
    }

    /**
     * @throws Throwable
     */
    public function update(
        UpdateMapRouteSolarsystemRequest $request,
        MapRouteSolarsystem $mapRouteSolarsystem,
        UpdateMapRouteSolarsystemAction $action
    ): RedirectResponse {
        $action->handle($mapRouteSolarsystem, $request->validated());

        return back()->notify('Route solarsystem updated!', message: 'Your route solarsystem has been updated successfully.');
    }

    /**
     * @throws Throwable
     */
    public function destroy(MapRouteSolarsystem $mapRouteSolarsystem, DeleteMapRouteSolarsystemAction $action): RedirectResponse
    {
        Gate::authorize('delete', $mapRouteSolarsystem);

        $action->handle($mapRouteSolarsystem);

        return back()->notify('Route solarsystem deleted!', message: 'Your route solarsystem has been deleted successfully.');
    }
}
