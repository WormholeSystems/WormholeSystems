<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Audit;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin Audit
 */
final class AuditResource extends JsonResource
{
    /**
     * @throws Throwable
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'tags' => $this->tags,
            'event' => $this->event,
            'new_values' => $this->new_values,
            'old_values' => $this->old_values,
            'character_id' => $this->character_id,
            'character' => $this->character->toResource(CharacterResource::class),
            'created_at' => $this->created_at,
        ];
    }
}
