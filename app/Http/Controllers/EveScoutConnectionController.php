<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\EveScout\AddEveScoutConnectionToMapAction;
use App\Http\Requests\StoreEveScoutConnectionRequest;
use App\Models\Solarsystem;
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

        $action->handle($request->map, Solarsystem::query()->firstWhere('name', $validated['solarsystem_name']));

        return back()->notify(
            'EVE Scout connections added!',
            'All connections have been added to your map.'
        );

    }
}
