<?php

declare(strict_types=1);

namespace App\Features;

use App\Models\MapUserSetting;
use App\Models\User;
use Illuminate\Support\Facades\Session;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;

final readonly class MapSettingsFeature implements ProvidesInertiaProperties
{
    private MapUserSetting $settings;

    public function __construct(
        private User $user,
        private int $map_id,
    ) {
        $this->settings = $this->user->mapUserSettings()->firstOrCreate([
            'map_id' => $this->map_id,
        ]);
    }

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'map_user_settings' => $this->settings->toResource(...),
            'ignored_systems' => fn () => Session::get('ignored_systems', []),
        ];
    }

    public function getSettings(): MapUserSetting
    {
        return $this->settings;
    }
}
