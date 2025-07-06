<?php

namespace App\Http\Controllers;

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapConnections\DeleteMapConnectionAction;
use App\Actions\MapConnections\UpdateMapConnectionAction;
use App\Http\Requests\StoreMapConnectionRequest;
use App\Http\Requests\UpdateMapConnectionRequest;
use App\Models\MapConnection;
use Illuminate\Http\RedirectResponse;

class MapConnectionController extends Controller
{
    public function store(StoreMapConnectionRequest $request, CreateMapConnectionAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back();
    }

    public function destroy(MapConnection $mapConnection, DeleteMapConnectionAction $action): RedirectResponse
    {
        $action->handle($mapConnection);

        return back();
    }

    public function update(UpdateMapConnectionRequest $request, MapConnection $mapConnection, UpdateMapConnectionAction $action): RedirectResponse
    {
        $action->handle($mapConnection, $request->validated());

        return back();
    }
}
