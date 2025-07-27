<?php

namespace App\Services;

use App\Models\Map;

readonly class RouteOptions
{
    public function __construct(
        public bool $allowEol = true,
        public bool $allowCrit = false,
        public bool $allowEveScout = false,
        public ?Map $map = null,
        public array $ignoredSystems = [], // Session-based system exclusions
    ) {}
}
