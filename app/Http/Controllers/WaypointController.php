<?php

namespace App\Http\Controllers;

use App\Actions\SetWaypointAction;
use App\Http\Requests\SetWaypointRequest;
use Illuminate\Http\RedirectResponse;

class WaypointController extends Controller
{
    public function store(SetWaypointRequest $request, SetWaypointAction $setWaypointAction): RedirectResponse
    {
        $setWaypointAction->handle($request->validated());

        return back()->notify('Waypoint set', message: 'Your waypoint has been set successfully.');
    }
}
