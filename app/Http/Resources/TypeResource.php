<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Type;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Type
 */
final class TypeResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'group_id' => $this->group_id,
            'description' => $this->description,
            'icon_id' => $this->icon_id,
            'volume' => $this->volume,
        ];
    }
}
