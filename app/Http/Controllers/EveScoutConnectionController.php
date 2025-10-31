<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\EveScout\AddEveScoutConnectionToMapAction;
use App\Http\Requests\StoreEveScoutConnectionRequest;
use Illuminate\Http\RedirectResponse;
use Throwable;

final class EveScoutConnectionController extends Controller
{
    /**
     * @throws Throwable
     */
    public function store(
        StoreEveScoutConnectionRequest $request,
        AddEveScoutConnectionToMapAction $action
    ): RedirectResponse {
        $validated = $request->validated();

        $action->handle($request->map, $validated['special_system_id']);

        return back()->notify(
            'EVE Scout connections added!',
            'All connections have been added to your map.'
        );

    }
}
