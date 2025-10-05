<?php

declare(strict_types=1);

namespace App\DTO;

final readonly class RouteDistance
{
    public function __construct(
        public int $targetSystemId,
        public float $cost,
        public int $jumps,
    ) {}
}
