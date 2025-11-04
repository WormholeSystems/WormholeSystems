<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Sovereignty;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Sovereignty
 */
final class SovereigntyResource extends JsonResource
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
            'corporation' => $this->getCorporation(),
            'alliance' => $this->getAlliance(),
            'faction' => $this->getFaction(),
        ];
    }

    private function getCorporation(): ?array
    {
        if ($this->corporation === null) {
            return null;
        }

        return [
            'id' => $this->corporation->id,
            'name' => $this->corporation->name,
            'ticker' => $this->corporation->ticker,
        ];
    }

    private function getAlliance(): ?array
    {
        if ($this->alliance === null) {
            return null;
        }

        return [
            'id' => $this->alliance->id,
            'name' => $this->alliance->name,
            'ticker' => $this->alliance->ticker,
        ];
    }

    private function getFaction(): ?array
    {
        if ($this->faction === null) {
            return null;
        }

        return [
            'id' => $this->faction->id,
            'name' => $this->faction->name,
        ];
    }
}
