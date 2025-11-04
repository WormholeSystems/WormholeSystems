<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Resources\MapInfoResource;
use App\Models\Map;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

final class MapSettingsController extends Controller
{
    /**
     * Show the general map settings page
     *
     * @throws Throwable
     */
    public function show(Map $map): Response
    {
        Gate::authorize('delete', $map);

        return Inertia::render('maps/settings/ShowGeneral', [
            'map' => $map->toResource(MapInfoResource::class),
            'is_owner' => Gate::allows('delete', $map),
            'has_write_access' => Gate::allows('update', $map),
        ]);
    }
}
