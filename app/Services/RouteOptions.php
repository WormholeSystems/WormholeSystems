<?php

namespace App\Services;

use App\Enums\MassStatus;
use App\Models\Map;

readonly class RouteOptions
{
    public function __construct(
        public bool $allowEol,
        public MassStatus $massStatus,
        public bool $allowEveScout = false,
        public ?Map $map = null,
        public array $ignoredSystems = [], // Session-based system exclusions
    ) {}
}
