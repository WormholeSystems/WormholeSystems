<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\User;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin User
 */
final class UserResource extends JsonResource
{
    /**
     * @throws Throwable
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'active_character' => $this->active_character?->load('esiScopes')->toResource(CharacterResource::class),
            'characters' => $this->characters->toResourceCollection(CharacterResource::class),
        ];
    }
}
