<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Models\Map;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Session;

final class MapScopeController extends Controller
{
    public function __construct() {}

    /**
     * Grant scopes and redirect back to the specific map
     */
    public function show(Request $request, Map $map): RedirectResponse
    {
        // Store the map-specific redirect URL
        Session::put('redirect_to', route('maps.show', $map));

        return to_route('eve.show', [
            'add_to_account' => true,
            'scopes' => $request->input('scopes'),
        ]);
    }
}
