<?php

namespace App\Http\Resources;

use App\Models\MapUserSetting;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapUserSetting
 */
class MapUserSettingResource extends JsonResource
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
        ];
    }
}
