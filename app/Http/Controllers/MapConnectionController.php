<?php

namespace App\Http\Controllers;

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapConnections\DeleteMapConnectionAction;
use App\Http\Requests\StoreMapConnectionRequest;
use App\Models\MapConnection;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;

class MapConnectionController extends Controller
{
    public function store(StoreMapConnectionRequest $request, CreateMapConnectionAction $action): RedirectResponse
    {
        $request->validated();
        $action->handle($request->fromMapSolarsystem, $request->toMapSolarsystem);

        return back();
    }

    public function destroy(Request $request, MapConnection $mapConnection, DeleteMapConnectionAction $action): RedirectResponse
    {
        $action->handle($mapConnection);

        return back();
    }
}
