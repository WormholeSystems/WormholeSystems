<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Events\Maps\MapUpdatedEvent;
use App\Models\Map;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;
use Illuminate\Validation\Rule;

final class HomeSystemController extends Controller
{
    public function store(Request $request, Map $map): RedirectResponse
    {
        Gate::authorize('update', $map);

        $validated = $request->validate([
            'map_solarsystem_id' => ['nullable', 'integer', Rule::exists('map_solarsystems', 'id')->where('map_id', $map->id)],
        ]);

        $map->update(['home_solarsystem_id' => $validated['map_solarsystem_id']]);

        broadcast(new MapUpdatedEvent($map))->toOthers();

        return back();
    }
}
