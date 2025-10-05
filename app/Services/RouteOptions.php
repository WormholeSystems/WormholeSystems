<?php

declare(strict_types=1);

namespace App\Services;

use App\Enums\MassStatus;
use App\Enums\RoutePreference;
use App\Models\Map;

final readonly class RouteOptions
{
    public function __construct(
        public bool $allowEol = false,
        public MassStatus $massStatus = MassStatus::Reduced,
        public bool $allowEveScout = false,
        public ?Map $map = null,
        public array $ignoredSystems = [], // Session-based system exclusions
        public RoutePreference $routePreference = RoutePreference::Shorter,
        public int $securityPenalty = 50, // 0-100, default 50
    ) {}
}
