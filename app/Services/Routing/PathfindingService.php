<?php

declare(strict_types=1);

namespace App\Services\Routing;

use App\DTO\RouteDistance;
use App\Services\RouteNode;
use App\Services\RouteOptions;
use SplPriorityQueue;

final readonly class PathfindingService
{
    public function __construct(
        private RouteCostCalculator $costCalculator,
    ) {}

    /**
     * Find the shortest path using A* algorithm with cost-based routing
     */
    public function findShortestPath(int $start, int $end, array $connections, RouteOptions $options): array
    {
        if ($start === $end) {
            return [$start];
        }

        if ($this->isDirectlyConnected($start, $end, $connections)) {
            return [$start, $end];
        }

        $queue = new SplPriorityQueue;
        $visited = [];
        $distances = [$start => 0];
        $bestPathCost = PHP_INT_MAX;

        $queue->insert(new RouteNode($start, 0, [$start]), 0);

        while (! $queue->isEmpty()) {
            $current = $queue->extract();

            if (isset($visited[$current->solarsystemId])) {
                continue;
            }

            // Early termination: if current cost is already >= best found path, skip
            if ($current->distance >= $bestPathCost) {
                continue;
            }

            $visited[$current->solarsystemId] = true;

            if ($current->solarsystemId === $end) {
                return $current->path;
            }

            $neighbors = $connections[$current->solarsystemId] ?? [];

            foreach ($neighbors as $neighbor) {
                if (isset($visited[$neighbor])) {
                    continue;
                }

                $edgeCost = $this->costCalculator->calculateEdgeCost($current->solarsystemId, $neighbor, $options);
                $newCost = $current->distance + $edgeCost;

                // Skip if this path is already more expensive than our best found path
                if ($newCost >= $bestPathCost) {
                    continue;
                }

                if (! isset($distances[$neighbor]) || $newCost < $distances[$neighbor]) {
                    $distances[$neighbor] = $newCost;

                    $newPath = [...$current->path, $neighbor];
                    $heuristic = $this->costCalculator->calculateHeuristic();

                    $queue->insert(
                        new RouteNode($neighbor, $newCost, $newPath, $heuristic),
                        -($newCost + $heuristic)
                    );
                }
            }
        }

        return [];
    }

    /**
     * Find distances to multiple target systems using Dijkstra-like approach
     *
     * @param  int  $start  Starting system ID
     * @param  int[]  $targets  Target system IDs
     * @param  array<int, int[]>  $connections  Connection graph
     * @param  int[]  $ignoredSystems  Systems to exclude from routing
     * @param  RouteOptions  $options  Routing options
     * @return RouteDistance[] Array of RouteDistance objects
     */
    public function findDistancesToMultipleTargets(
        int $start,
        array $targets,
        array $connections,
        array $ignoredSystems,
        RouteOptions $options
    ): array {
        if ($targets === []) {
            return [];
        }

        // Convert targets to hash map for O(1) lookups
        $targetMap = array_flip($targets);
        $distances = [];
        $jumps = [];
        $visited = [];
        $foundTargets = [];
        $queue = new SplPriorityQueue;

        // Start from the origin
        $queue->insert(['id' => $start, 'cost' => 0, 'jumps' => 0], 0);
        $distances[$start] = 0;
        $jumps[$start] = 0;

        // If start is a target, record it
        if (isset($targetMap[$start])) {
            $foundTargets[$start] = ['cost' => 0, 'jumps' => 0];
        }

        while (! $queue->isEmpty()) {
            $current = $queue->extract();
            $currentId = $current['id'];
            $currentCost = $current['cost'];
            $currentJumps = $current['jumps'];

            // Skip if already visited
            if (isset($visited[$currentId])) {
                continue;
            }

            $visited[$currentId] = true;

            // If this is a target and we haven't recorded it yet, record the optimal cost and jumps
            if (isset($targetMap[$currentId]) && ! isset($foundTargets[$currentId])) {
                $foundTargets[$currentId] = ['cost' => $currentCost, 'jumps' => $currentJumps];
            }

            // Early termination if we found all targets
            if (count($foundTargets) >= count($targets)) {
                break;
            }

            $neighbors = $connections[$currentId] ?? [];

            foreach ($neighbors as $neighbor) {
                // Skip if already visited or is an ignored system
                if (isset($visited[$neighbor])) {
                    continue;
                }
                if (in_array($neighbor, $ignoredSystems, true)) {
                    continue;
                }

                $edgeCost = $this->costCalculator->calculateEdgeCost($currentId, $neighbor, $options);
                $newCost = $currentCost + $edgeCost;
                $newJumps = $currentJumps + 1;

                // Only update if we found a better path (by cost)
                if (! isset($distances[$neighbor]) || $newCost < $distances[$neighbor]) {
                    $distances[$neighbor] = $newCost;
                    $jumps[$neighbor] = $newJumps;
                    $queue->insert(['id' => $neighbor, 'cost' => $newCost, 'jumps' => $newJumps], -$newCost);
                }
            }
        }

        // Convert to DTOs
        $results = [];
        foreach ($targets as $target) {
            if (isset($foundTargets[$target])) {
                $results[] = new RouteDistance(
                    targetSystemId: $target,
                    cost: $foundTargets[$target]['cost'],
                    jumps: $foundTargets[$target]['jumps']
                );
            } else {
                // Unreachable targets
                $results[] = new RouteDistance(
                    targetSystemId: $target,
                    cost: PHP_FLOAT_MAX,
                    jumps: PHP_INT_MAX
                );
            }
        }

        return $results;
    }

    /**
     * Check if two systems are directly connected
     */
    private function isDirectlyConnected(int $from, int $to, array $connections): bool
    {
        return isset($connections[$from]) && in_array($to, $connections[$from], true);
    }
}
