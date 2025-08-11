<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\ShipHistory;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin ShipHistory
 */
final class ShipHistoryResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     *
     * @throws Throwable
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'ship_id' => $this->ship_id,
            'ship_type_id' => $this->ship_type_id,
            'character_id' => $this->character_id,
            'name' => $this->name,
            'ship_type' => $this->shipType->toResource(TypeResource::class),
            'character' => $this->character->toResource(CharacterResource::class),
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
        ];
    }
}
