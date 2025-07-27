<?php

namespace App\Services;

readonly class RouteNode
{
    public function __construct(
        public int $solarsystemId,
        public float $distance,
        public array $path,
        public float $heuristic = 0.0,
    ) {}
}
