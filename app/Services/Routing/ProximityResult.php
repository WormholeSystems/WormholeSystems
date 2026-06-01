<?php

declare(strict_types=1);

namespace App\Services\Routing;

final readonly class ProximityResult
{
    /**
     * @param  int[]  $route  Ordered solarsystem ids from the matched origin to the target.
     */
    public function __construct(
        public int $jumps,
        public int $matchedOriginSolarsystemId,
        public array $route,
    ) {}
}
