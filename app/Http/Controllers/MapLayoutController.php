<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Map\UpdateMapAction;
use App\Http\Requests\UpdateMapLayoutRequest;
use App\Models\Map;
use Illuminate\Http\RedirectResponse;

final class MapLayoutController extends Controller
{
    /**
     * Switch the map's layout mode. It is a per-map setting shared by every viewer, so the
     * change is broadcast (via UpdateMapAction) to refresh everyone's map.
     */
    public function update(UpdateMapLayoutRequest $request, Map $map, UpdateMapAction $action): RedirectResponse
    {
        $action->handle($map, $request->validated());

        return back();
    }
}
