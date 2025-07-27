<?php

namespace App\Http\Controllers;

use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Session;

class IgnoreListController extends Controller
{
    /**
     * Add a system to the ignore list
     */
    public function store(Request $request): RedirectResponse
    {
        $systemId = $request->integer('solarsystem_id');
        $ignoredSystems = Session::get('ignored_systems', []);

        if (! in_array($systemId, $ignoredSystems, true)) {
            $ignoredSystems[] = $systemId;
            Session::put('ignored_systems', $ignoredSystems);
        }

        return back()->notify(
            'System ignored',
            'The system has been added to your ignore list.'
        );
    }

    /**
     * Remove a system from the ignore list
     */
    public function destroy(int $solarsystem_id): RedirectResponse
    {
        $ignoredSystems = Session::get('ignored_systems', []);
        $ignoredSystems = array_values(array_filter($ignoredSystems, fn ($id): bool => $id !== $solarsystem_id));

        Session::put('ignored_systems', $ignoredSystems);

        return back()->notify(
            'System unignored',
            'The system has been removed from your ignore list.'
        );
    }

    /**
     * Clear all ignored systems
     */
    public function destroyAll(): RedirectResponse
    {
        Session::forget('ignored_systems');

        return back()->notify(
            'Ignore list cleared',
            'All systems have been removed from your ignore list.'
        );
    }
}
