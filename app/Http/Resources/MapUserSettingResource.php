<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Enums\MapBackgroundMode;
use App\Models\MapUserSetting;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Facades\Storage;

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
            'id' => $this->id ?? null,
            'user_id' => $this->user_id ?? null,
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
            'auto_confirm_signatures' => $this->auto_confirm_signatures ?? false,
            'first_layer_nato_alias' => $this->first_layer_nato_alias ?? false,
            'suggest_alias_enabled' => $this->suggest_alias_enabled,
            'concat_alias_disabled' => $this->concat_alias_disabled,
            'copy_bookmark_enabled' => $this->copy_bookmark_enabled,
            'layout_breakpoints' => $this->layout_breakpoints,
            'hidden_cards' => $this->hidden_cards ?? [],
            'show_threat_level' => $this->show_threat_level,
            'show_statics_first' => $this->show_statics_first,
            'is_archived' => $this->is_archived ?? false,
            'background_image_url' => $this->background_image_path
                ? Storage::disk('public')->url($this->background_image_path)
                : null,
            'background_image_mode' => $this->background_image_mode ?? MapBackgroundMode::Grid,
        ];
    }
}
