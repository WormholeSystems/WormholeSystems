<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\DTO\MaintainerCharacterStat;
use App\DTO\MaintainerEntry;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * Deliberately minimal -- {id, name} per character, no point breakdown. The settings
 * page needs the per-alt contributions and must consume the MaintainerEntry DTOs (or a
 * richer resource) directly rather than reuse this one.
 *
 * @mixin MaintainerEntry
 */
final class MaintainerEntryResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array{position: int, points: int, user_id: int|null, characters: list<array{id: int, name: string}>}
     */
    public function toArray(Request $request): array
    {
        return [
            'position' => $this->position,
            'points' => $this->points,
            'user_id' => $this->user_id,
            'characters' => array_map(
                fn (MaintainerCharacterStat $character): array => [
                    'id' => $character->character_id,
                    'name' => $character->character_name,
                ],
                $this->characters,
            ),
        ];
    }
}
