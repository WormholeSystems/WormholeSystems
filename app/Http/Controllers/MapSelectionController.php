<?php

namespace App\Http\Controllers;

use App\Actions\MapSelection\DeleteMapSelectionAction;
use App\Actions\MapSelection\UpdateMapSelectionAction;
use App\Http\Requests\DeleteMapSelectionRequest;
use App\Http\Requests\UpdateMapSelectionRequest;
use Illuminate\Http\RedirectResponse;
use Throwable;

class MapSelectionController extends Controller
{
    /**
     * @throws Throwable
     */
    public function update(UpdateMapSelectionRequest $request, UpdateMapSelectionAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back();
    }

    /**
     * @throws Throwable
     */
    public function destroy(DeleteMapSelectionRequest $request, DeleteMapSelectionAction $action): RedirectResponse
    {
        $action->handle($request->validated()['map_solarsystem_ids']);

        return back()->notify(
            'Selection deleted!',
            'You successfully deleted the selected items!'
        );
    }
}
