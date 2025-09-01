<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapConnections\DeleteMapConnectionAction;
use App\Actions\MapConnections\UpdateMapConnectionAction;
use App\Data\MapConnectionData;
use App\Http\Requests\StoreMapConnectionRequest;
use App\Http\Requests\UpdateMapConnectionRequest;
use App\Models\MapConnection;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;

final class MapConnectionController extends Controller
{
    public function store(StoreMapConnectionRequest $request, CreateMapConnectionAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back()->notify(
            'Connection created!',
            'You have successfully created a new map connection.'
        );
    }

    public function destroy(MapConnection $mapConnection, DeleteMapConnectionAction $action): RedirectResponse
    {
        Gate::authorize('delete', $mapConnection);

        $action->handle($mapConnection);

        return back()->notify(
            'Connection deleted!',
            'You have successfully deleted the map connection.'
        );
    }

    public function update(UpdateMapConnectionRequest $request, MapConnection $mapConnection, UpdateMapConnectionAction $action): RedirectResponse
    {
        Gate::authorize('update', $mapConnection);

        $action->handle($mapConnection, MapConnectionData::from($request->validated()));

        return back()->notify(
            'Connection updated!',
            'You have successfully updated the map connection.'
        );
    }
}
