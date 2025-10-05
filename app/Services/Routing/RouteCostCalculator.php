<?php

declare(strict_types=1);

namespace App\Services\Routing;

use App\Enums\RoutePreference;
use App\Services\RouteOptions;

final readonly class RouteCostCalculator
{
    public function __construct(
        private array $solarsystems,
    ) {}

    /**
     * Calculate the cost of traversing from one system to another
     */
    public function calculateEdgeCost(int $from, int $to, RouteOptions $options): float
    {
        $fromSystem = $this->solarsystems[$from] ?? null;
        $toSystem = $this->solarsystems[$to] ?? null;

        if (! $fromSystem || ! $toSystem) {
            return 1; // Default cost if system data not available
        }

        return match ($options->routePreference) {
            RoutePreference::Shorter => $this->calculateShorterCost(),
            RoutePreference::Safer => $this->calculateSaferCost($toSystem['security'], $options->securityPenalty),
            RoutePreference::LessSecure => $this->calculateLessSecureCost($toSystem['security'], $options->securityPenalty),
        };
    }

    /**
     * Calculate heuristic for A* algorithm
     * Returns 0 to ensure optimal routes (A* with h=0 is equivalent to Dijkstra)
     */
    public function calculateHeuristic(): float
    {
        // Heuristic should return 0 to ensure optimal routes
        // A* with h=0 is equivalent to Dijkstra's algorithm
        return 0;
    }

    /**
     * Shorter route: Always 1 (minimum jumps)
     */
    private function calculateShorterCost(): float
    {
        return 1;
    }

    /**
     * Safer route: Prefer high-sec, penalize low/null-sec
     * Based on EVE Online's safer route formula
     */
    private function calculateSaferCost(float $security, int $securityPenalty): float
    {
        $penaltyCost = exp(0.15 * $securityPenalty);
        if ($security <= 0.0) {
            // Null sec: heavily penalized
            return 2 * $penaltyCost;
        }

        if ($security < 0.45) {
            // Low sec: penalized
            return $penaltyCost;
        }

        // High sec: preferred
        return 0.90;

    }

    /**
     * Less secure route: Prefer low-sec, penalize high/null-sec
     * Based on EVE Online's less-secure route formula
     */
    private function calculateLessSecureCost(float $security, int $securityPenalty): float
    {
        $penaltyCost = exp(0.15 * $securityPenalty);
        if ($security <= 0.0) {
            // Null sec: heavily penalized
            return 2 * $penaltyCost;
        }

        if ($security < 0.45) {
            // Low sec: preferred
            return 0.90;
        }

        // High sec: penalized
        return $penaltyCost;

    }
}
