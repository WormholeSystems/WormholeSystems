<?php

namespace App\Http\Controllers;

use App\Actions\MapSolarsystem\DeleteMapSolarsystemAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Actions\MapSolarsystem\UpdateMapSolarsystemAction;
use App\Http\Requests\StoreMapSolarsystemRequest;
use App\Http\Requests\UpdateMapSolarsystemRequest;
use App\Models\MapSolarsystem;
use Illuminate\Http\RedirectResponse;

class MapSolarsystemController extends Controller
{
    public function store(StoreMapSolarsystemRequest $request, StoreMapSolarsystemAction $action): RedirectResponse
    {
        $action->handle($request->map, $request->validated());

        return back();
    }

    public function update(UpdateMapSolarsystemRequest $request, MapSolarsystem $mapSolarsystem, UpdateMapSolarsystemAction $action): RedirectResponse
    {
        $action->handle($mapSolarsystem, $request->validated());

        return back();
    }

    public function destroy(MapSolarsystem $mapSolarsystem, DeleteMapSolarsystemAction $action): RedirectResponse
    {
        $action->handle($mapSolarsystem);

        return back();
    }
}
