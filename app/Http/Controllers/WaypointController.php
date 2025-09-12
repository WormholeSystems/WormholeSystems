<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\SetWaypointAction;
use App\DTO\CTA;
use App\Http\Requests\SetWaypointRequest;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Http\RedirectResponse;

final class WaypointController extends Controller
{
    /**
     * @throws ConnectionException
     */
    public function store(SetWaypointRequest $request, SetWaypointAction $setWaypointAction): RedirectResponse
    {
        $validated = $request->validated();

        if ($request->character->esiTokens()->hasWaypointScopes()->doesntExist()) {
            return back()->notify(
                'Additional Permission Required',
                message: 'To set waypoints, you need to grant waypoint permissions for this character.',
                type: 'warning',
                action: CTA::make()->title('Manage permission')->url(route('scopes.index'))
            );
        }

        $setWaypointAction->handle($request->character, $validated);

        return back()->notify('Waypoint set', message: 'Your waypoint has been set successfully.');
    }
}
