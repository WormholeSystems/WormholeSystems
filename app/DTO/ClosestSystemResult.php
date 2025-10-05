<?php

declare(strict_types=1);

namespace App\DTO;

use App\Models\Solarsystem;

final readonly class ClosestSystemResult
{
    public function __construct(
        public Solarsystem $solarsystem,
        public float $cost,
        public int $jumps,
    ) {}

    public function toArray(): array
    {
        return [
            'solarsystem' => $this->solarsystem->toResource(),
            'cost' => $this->cost,
            'jumps' => $this->jumps,
            'distance' => $this->jumps, // Backwards compatibility
        ];
    }
}
