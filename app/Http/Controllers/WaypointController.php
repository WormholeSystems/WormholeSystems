<?php

namespace App\Http\Controllers;

use App\Actions\SetWaypointAction;
use App\Http\Requests\SetWaypointRequest;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Http\RedirectResponse;
use NicolasKion\Esi\Enums\EsiScope;

class WaypointController extends Controller
{
    /**
     * @throws ConnectionException
     */
    public function store(SetWaypointRequest $request, SetWaypointAction $setWaypointAction): RedirectResponse
    {
        $validated = $request->validated();

        if (! $request->character->esiTokens()->whereRelation('esiScopes', 'name', EsiScope::WriteWaypoint)->exists()) {
            return back()->notify('Missing ESI scope', message: 'Please reauthenticate your character to set waypoints.', type: 'error');
        }

        $setWaypointAction->handle($request->character, $validated);

        return back()->notify('Waypoint set', message: 'Your waypoint has been set successfully.');
    }
}
