<?php

declare(strict_types=1);

namespace App\Services\Routing;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\File;

/**
 * Server-side port of the routing graph traversal in resources/js/routing/algorithm.ts.
 *
 * For proximity alerts we only need hop counts, so this performs an unweighted
 * multi-source breadth-first search over the static stargate graph merged with the
 * map's live wormhole connections, rather than the frontend's weighted Dijkstra.
 */
final class MapProximityPathfinder
{
    /**
     * Zarzakh uses Jovian stargates that require special access - never route through it.
     */
    private const int ZARZAKH_SYSTEM_ID = 30100000;

    /**
     * Find the fewest jumps from the nearest origin system to the target system.
     *
     * @param  int[]  $originSolarsystemIds  Solarsystem ids currently placed on the map.
     * @param  array<int, array{0: int, 1: int}>  $wormholeEdges  Pairs of [fromSolarsystemId, toSolarsystemId].
     * @param  int[]  $ignoredSolarsystemIds  Systems that must not be traversed.
     * @param  int  $maxJumps  Maximum hops to search before giving up.
     */
    public function nearest(
        array $originSolarsystemIds,
        int $targetSolarsystemId,
        array $wormholeEdges,
        array $ignoredSolarsystemIds,
        int $maxJumps,
    ): ?ProximityResult {
        if ($originSolarsystemIds === []) {
            return null;
        }

        $adjacency = $this->buildAdjacency($wormholeEdges);
        $excluded = $this->buildExcludedSet($ignoredSolarsystemIds, $originSolarsystemIds, $targetSolarsystemId);

        /** @var array<int, int> $distance */
        $distance = [];
        /** @var array<int, int|null> $previous */
        $previous = [];
        /** @var array<int, int> $originOf */
        $originOf = [];
        /** @var int[] $queue */
        $queue = [];

        foreach ($originSolarsystemIds as $origin) {
            if (isset($distance[$origin])) {
                continue;
            }

            if ($origin === $targetSolarsystemId) {
                return new ProximityResult(0, $origin, [$origin]);
            }

            $distance[$origin] = 0;
            $previous[$origin] = null;
            $originOf[$origin] = $origin;
            $queue[] = $origin;
        }

        $head = 0;
        while ($head < count($queue)) {
            $current = $queue[$head++];
            $currentDistance = $distance[$current];

            if ($currentDistance >= $maxJumps) {
                continue;
            }

            foreach ($adjacency[$current] ?? [] as $neighbor) {
                if (isset($distance[$neighbor])) {
                    continue;
                }
                if (isset($excluded[$neighbor])) {
                    continue;
                }
                $distance[$neighbor] = $currentDistance + 1;
                $previous[$neighbor] = $current;
                $originOf[$neighbor] = $originOf[$current];

                if ($neighbor === $targetSolarsystemId) {
                    return new ProximityResult(
                        $distance[$neighbor],
                        $originOf[$neighbor],
                        $this->reconstructRoute($neighbor, $previous),
                    );
                }

                $queue[] = $neighbor;
            }
        }

        return null;
    }

    /**
     * @param  array<int, array{0: int, 1: int}>  $wormholeEdges
     * @return array<int, int[]>
     */
    private function buildAdjacency(array $wormholeEdges): array
    {
        $adjacency = $this->stargateAdjacency();

        foreach ($wormholeEdges as [$from, $to]) {
            $adjacency[$from][] = $to;
            $adjacency[$to][] = $from;
        }

        return $adjacency;
    }

    /**
     * The static k-space stargate graph, decoded once and cached as a symmetric adjacency map.
     *
     * @return array<int, int[]>
     */
    private function stargateAdjacency(): array
    {
        $path = resource_path('static/connections.json');

        return Cache::rememberForever('routing.stargate_adjacency.'.filemtime($path), function () use ($path): array {
            /** @var array<int|string, int[]> $raw */
            $raw = json_decode(File::get($path), true, flags: JSON_THROW_ON_ERROR);

            $adjacency = [];
            foreach ($raw as $from => $neighbors) {
                $fromId = (int) $from;
                foreach ($neighbors as $to) {
                    $toId = (int) $to;
                    $adjacency[$fromId][] = $toId;
                    $adjacency[$toId][] = $fromId;
                }
            }

            return $adjacency;
        });
    }

    /**
     * Build the set of systems that may not be traversed. Origins and the target are
     * always exempt, mirroring isIgnoredNode() in the frontend algorithm.
     *
     * @param  int[]  $ignoredSolarsystemIds
     * @param  int[]  $originSolarsystemIds
     * @return array<int, true>
     */
    private function buildExcludedSet(array $ignoredSolarsystemIds, array $originSolarsystemIds, int $targetSolarsystemId): array
    {
        $excluded = array_fill_keys($ignoredSolarsystemIds, true);
        $excluded[self::ZARZAKH_SYSTEM_ID] = true;

        unset($excluded[$targetSolarsystemId]);
        foreach ($originSolarsystemIds as $origin) {
            unset($excluded[$origin]);
        }

        return $excluded;
    }

    /**
     * @param  array<int, int|null>  $previous
     * @return int[]
     */
    private function reconstructRoute(int $targetId, array $previous): array
    {
        $path = [];
        $node = $targetId;

        while ($node !== null) {
            $path[] = $node;
            $node = $previous[$node] ?? null;
        }

        return array_reverse($path);
    }
}
