<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Map\UpdateMapAction;
use App\Http\Requests\UpdateMapBookmarkFormatRequest;
use App\Models\Map;
use Illuminate\Http\RedirectResponse;

final class MapBookmarkFormatController extends Controller
{
    /**
     * Update the connection bookmark name templates. They are a per-map setting shared by
     * every viewer, so the change is broadcast (via UpdateMapAction) to refresh everyone.
     */
    public function update(UpdateMapBookmarkFormatRequest $request, Map $map, UpdateMapAction $action): RedirectResponse
    {
        $action->handle($map, $request->validated());

        return back();
    }
}
