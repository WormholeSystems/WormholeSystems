<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\DTO\MaintainerCharacterStat;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MaintainerCharacterStat
 */
final class MaintainerCharacterStatResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array{character_id: int, character_name: string, user_id: int|null, nb_added: int, nb_edited: int, nb_deleted: int, points: int}
     */
    public function toArray(Request $request): array
    {
        return [
            'character_id' => $this->character_id,
            'character_name' => $this->character_name,
            'user_id' => $this->user_id,
            'nb_added' => $this->nb_added,
            'nb_edited' => $this->nb_edited,
            'nb_deleted' => $this->nb_deleted,
            'points' => $this->points,
        ];
    }
}
