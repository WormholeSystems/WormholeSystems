<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Requests\StoreMapBackgroundImageRequest;
use App\Models\Map;
use App\Models\MapUserSetting;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Storage;

final class MapBackgroundImageController extends Controller
{
    public function store(StoreMapBackgroundImageRequest $request, Map $map, #[CurrentUser] User $user): RedirectResponse
    {
        $settings = $user->mapUserSettings()->firstOrCreate(['map_id' => $map->id]);

        $this->deleteStoredImage($settings);

        $path = $request->file('background_image')->store("map-backgrounds/{$map->id}", 'public');

        $settings->update(['background_image_path' => $path]);

        return back();
    }

    public function destroy(Map $map, #[CurrentUser] User $user): RedirectResponse
    {
        abort_unless($user->can('view', $map), 403);

        $settings = $user->mapUserSettings()->firstOrCreate(['map_id' => $map->id]);

        $this->deleteStoredImage($settings);

        $settings->update(['background_image_path' => null]);

        return back();
    }

    private function deleteStoredImage(MapUserSetting $settings): void
    {
        if ($settings->background_image_path !== null) {
            Storage::disk('public')->delete($settings->background_image_path);
        }
    }
}
