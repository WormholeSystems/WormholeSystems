<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\MapUserSetting;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapUserSetting
 */
final class MapUserSettingResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'user_id' => $this->user_id,
            'map_id' => $this->map_id,
            'tracking_allowed' => $this->tracking_allowed,
            'is_tracking' => $this->is_tracking,
            'route_allow_lifetime_status' => $this->route_allow_lifetime_status,
            'route_allow_mass_status' => $this->route_allow_mass_status,
            'route_use_evescout' => $this->route_use_evescout,
            'route_preference' => $this->route_preference,
            'security_penalty' => $this->security_penalty,
            'killmail_filter' => $this->killmail_filter,
            'introduction_confirmed_at' => $this->introduction_confirmed_at?->toISOString(),
            'prompt_for_signature_enabled' => $this->prompt_for_signature_enabled,
            'layout_breakpoints' => $this->layout_breakpoints,
            'hidden_cards' => $this->hidden_cards ?? [],
        ];
    }
}
