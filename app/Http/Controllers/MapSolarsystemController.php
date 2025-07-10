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

        return back()->notify(
            'Solarsystem created!',
            'You have successfully created a new solarsystem on the map.'
        );
    }

    public function update(UpdateMapSolarsystemRequest $request, MapSolarsystem $mapSolarsystem, UpdateMapSolarsystemAction $action): RedirectResponse
    {
        $action->handle($mapSolarsystem, $request->validated());

        return back()->notify(
            'Solarsystem updated!',
            'You have successfully updated the solarsystem on the map.'
        );
    }

    public function destroy(MapSolarsystem $mapSolarsystem, DeleteMapSolarsystemAction $action): RedirectResponse
    {
        $action->handle($mapSolarsystem);

        return back()->notify(
            'Solarsystem deleted!',
            'You have successfully deleted the solarsystem from the map.'
        );
    }
}
