<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Resources\MapInfoResource;
use App\Models\Map;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Illuminate\Support\Str;
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
        Gate::authorize('manageAccess', $map);

        return Inertia::render('maps/settings/ShowGeneral', [
            'map' => $map->toResource(MapInfoResource::class),
            'is_owner' => Gate::allows('delete', $map),
            'permission' => $map->getUserPermission(auth()->user())?->value,
            'is_public' => $map->is_public,
            'share_token' => $map->share_token,
        ]);
    }

    public function togglePublic(Map $map): RedirectResponse
    {
        Gate::authorize('manageAccess', $map);

        $map->update(['is_public' => ! $map->is_public]);

        $status = $map->is_public ? 'enabled' : 'disabled';

        return back()->notify('Public access '.$status.'.', 'Public access for this map has been '.$status.'.');
    }

    public function generateShareToken(Map $map): RedirectResponse
    {
        Gate::authorize('manageAccess', $map);

        $map->update(['share_token' => Str::uuid()->toString()]);

        return back()->notify('Share link generated.', 'A new share link has been generated for this map.');
    }

    public function revokeShareToken(Map $map): RedirectResponse
    {
        Gate::authorize('manageAccess', $map);

        $map->update(['share_token' => null]);

        return back()->notify('Share link revoked.', 'The share link has been revoked.');
    }
}
