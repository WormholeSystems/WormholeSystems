<?php

declare(strict_types=1);

namespace App\Services;

use App\Enums\MassStatus;
use App\Models\Map;

final readonly class RouteOptions
{
    public function __construct(
        public bool $allowEol,
        public MassStatus $massStatus,
        public bool $allowEveScout = false,
        public ?Map $map = null,
        public array $ignoredSystems = [], // Session-based system exclusions
    ) {}
}
