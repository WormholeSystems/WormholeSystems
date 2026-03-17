<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Models\Map;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;

final class RallyPointController extends Controller
{
    public function store(Request $request, Map $map): RedirectResponse
    {
        Gate::authorize('update', $map);

        $validated = $request->validate([
            'solarsystem_id' => ['nullable', 'integer', 'exists:solarsystems,id'],
        ]);

        $map->update(['rally_solarsystem_id' => $validated['solarsystem_id']]);

        return back();
    }
}
