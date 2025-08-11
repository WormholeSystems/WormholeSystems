<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Alliance;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Alliance
 */
final class AllianceResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray($request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'ticker' => $this->ticker,
        ];
    }
}
