<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Killmail;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin Killmail
 */
final class KillmailResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     *
     * @throws Throwable
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'hash' => $this->hash,
            'solarsystem' => $this->solarsystem->toResource(SolarsystemResource::class),
            'time' => $this->time,
            'data' => $this->data,
            'zkb' => $this->zkb,
            'ship_type' => $this->shipType->toResource(TypeResource::class),
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
        ];
    }
}
