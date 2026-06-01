<?php

declare(strict_types=1);

use App\Services\Routing\MapProximityPathfinder;
use Illuminate\Support\Facades\File;

/**
 * The graph nodes below use small synthetic ids that never collide with the real
 * EVE solarsystem ids in connections.json, so the only edges are the wormhole
 * edges we provide - giving us a fully controlled graph.
 */
function pathfinder(): MapProximityPathfinder
{
    return new MapProximityPathfinder;
}

it('returns zero jumps when the target is an origin', function () {
    $result = pathfinder()->nearest([5], 5, [], [], 5);

    expect($result)->not->toBeNull()
        ->and($result->jumps)->toBe(0)
        ->and($result->matchedOriginSolarsystemId)->toBe(5)
        ->and($result->route)->toBe([5]);
});

it('finds the target across wormhole edges', function () {
    $edges = [[1, 2], [2, 3]];

    $result = pathfinder()->nearest([1], 3, $edges, [], 5);

    expect($result)->not->toBeNull()
        ->and($result->jumps)->toBe(2)
        ->and($result->route)->toBe([1, 2, 3]);
});

it('prefers a shorter wormhole shortcut when one exists', function () {
    $result = pathfinder()->nearest([1], 3, [[1, 2], [2, 3], [1, 3]], [], 5);

    expect($result->jumps)->toBe(1)
        ->and($result->route)->toBe([1, 3]);
});

it('returns the nearest of multiple origins', function () {
    $edges = [[1, 2], [2, 3], [10, 3]];

    $result = pathfinder()->nearest([1, 10], 3, $edges, [], 5);

    expect($result->jumps)->toBe(1)
        ->and($result->matchedOriginSolarsystemId)->toBe(10);
});

it('returns null when the target is further than max jumps', function () {
    $edges = [[1, 2], [2, 3], [3, 4]];

    expect(pathfinder()->nearest([1], 4, $edges, [], 2))->toBeNull()
        ->and(pathfinder()->nearest([1], 4, $edges, [], 3))->not->toBeNull();
});

it('does not route through ignored systems', function () {
    $edges = [[1, 2], [2, 3]];

    expect(pathfinder()->nearest([1], 3, $edges, [2], 5))->toBeNull();
});

it('never routes through Zarzakh', function () {
    $zarzakh = 30100000;
    $edges = [[1, $zarzakh], [$zarzakh, 2]];

    expect(pathfinder()->nearest([1], 2, $edges, [], 5))->toBeNull();
});

it('traverses real stargate connections', function () {
    /** @var array<int|string, int[]> $raw */
    $raw = json_decode(File::get(resource_path('static/connections.json')), true);

    $from = (int) array_key_first($raw);
    $to = (int) $raw[$from][0];

    $result = pathfinder()->nearest([$from], $to, [], [], 1);

    expect($result)->not->toBeNull()
        ->and($result->jumps)->toBe(1);
});
