<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\SetWaypointAction;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;

final class BulkWaypointController extends Controller
{
    /**
     * @throws ConnectionException
     */
    public function store(Request $request, #[CurrentUser] User $user, SetWaypointAction $setWaypointAction): RedirectResponse
    {
        $validated = $request->validate([
            'destination_id' => ['required', 'integer', 'exists:solarsystems,id'],
            'clear_other_waypoints' => ['sometimes', 'boolean'],
            'add_to_beginning' => ['sometimes', 'boolean'],
        ]);

        $characters = $user->characters()->whereHas('esiTokens', fn ($q) => $q->hasWaypointScopes())->get();

        foreach ($characters as $character) {
            $setWaypointAction->handle($character, $validated);
        }

        $count = $characters->count();

        return back()->notify('Waypoints set', message: sprintf('Waypoint set for %d character%s.', $count, $count === 1 ? '' : 's'));
    }
}
