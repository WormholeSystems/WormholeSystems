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
    /** @var array<string, mixed> */
    public const array DEFAULTS = [
        'tracking_allowed' => false,
        'is_tracking' => false,
        'route_allow_lifetime_status' => 'critical',
        'route_allow_mass_status' => 'reduced',
        'route_use_evescout' => false,
        'route_preference' => 'shorter',
        'security_penalty' => 50,
        'killmail_filter' => 'all',
        'prompt_for_signature_enabled' => false,
        'layout_breakpoints' => null,
        'hidden_cards' => null,
    ];

    private MapUserSetting $settings;

    public function __construct(
        private ?User $user,
        private int $map_id,
    ) {
        $this->settings = $this->resolveSettings();
    }

    public static function sessionKey(int $mapId): string
    {
        return "map_settings.{$mapId}";
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

    private function resolveSettings(): MapUserSetting
    {
        $sessionKey = self::sessionKey($this->map_id);

        // Authenticated → always load from DB (source of truth), cache in session
        if ($this->user instanceof User) {
            $settings = $this->user->mapUserSettings()->firstOrCreate([
                'map_id' => $this->map_id,
            ]);

            Session::put($sessionKey, $settings->attributesToArray());

            return $settings;
        }

        // Guest → session is the primary store
        $sessionData = Session::get($sessionKey);

        if (is_array($sessionData)) {
            return $this->hydrateFromSession($sessionData);
        }

        $defaults = array_merge(self::DEFAULTS, ['map_id' => $this->map_id]);
        Session::put($sessionKey, $defaults);

        return $this->hydrateFromSession($defaults);
    }

    /**
     * @param  array<string, mixed>  $data
     */
    private function hydrateFromSession(array $data): MapUserSetting
    {
        $setting = new MapUserSetting();
        $setting->forceFill($data);

        return $setting;
    }
}
