<?php

declare(strict_types=1);

use App\Actions\MapConnections\CleanStaleMapConnectionsAction;
use App\Enums\LifetimeStatus;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\User;

use function Pest\Laravel\actingAs;

function pinnedSystem(Map $map, int $solarsystemId): MapSolarsystem
{
    $system = placeMapSolarsystem($map, $solarsystemId);
    $system->update(['pinned' => true]);

    return $system;
}

function staleConnection(Map $map, MapSolarsystem $from, MapSolarsystem $to): MapConnection
{
    return MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        'lifetime' => LifetimeStatus::Critical,
        'lifetime_updated_at' => now()->subHours(2),
    ]);
}

function freshConnection(Map $map, MapSolarsystem $from, MapSolarsystem $to): MapConnection
{
    return MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        'lifetime' => LifetimeStatus::Healthy,
        'lifetime_updated_at' => now()->subHours(2),
    ]);
}

it('removes a stale connection and its now-orphaned system', function () {
    $map = Map::factory()->create();
    $anchor = pinnedSystem($map, 30004001);
    $leaf = placeMapSolarsystem($map, 30004002);
    $connection = staleConnection($map, $anchor, $leaf);

    $count = app(CleanStaleMapConnectionsAction::class)->handle($map);

    expect($count)->toBe(1)
        ->and(MapConnection::find($connection->id))->toBeNull()
        ->and(MapSolarsystem::find($leaf->id))->toBeNull()
        ->and(MapSolarsystem::find($anchor->id))->not->toBeNull();
});

it('cascades the entire chain left dangling by a stale connection', function () {
    $map = Map::factory()->create();
    $anchor = pinnedSystem($map, 30004001);
    $b = placeMapSolarsystem($map, 30004002);
    $c = placeMapSolarsystem($map, 30004003);

    $stale = staleConnection($map, $anchor, $b);
    $fresh = freshConnection($map, $b, $c);

    app(CleanStaleMapConnectionsAction::class)->handle($map);

    expect(MapConnection::find($stale->id))->toBeNull()
        ->and(MapConnection::find($fresh->id))->toBeNull()
        ->and(MapSolarsystem::find($b->id))->toBeNull()
        ->and(MapSolarsystem::find($c->id))->toBeNull()
        ->and(MapSolarsystem::find($anchor->id))->not->toBeNull();
});

it('keeps a chain that is still reachable from another anchor', function () {
    $map = Map::factory()->create();
    $anchor = pinnedSystem($map, 30004001);
    $other = pinnedSystem($map, 30004002);
    $b = placeMapSolarsystem($map, 30004003);

    $stale = staleConnection($map, $anchor, $b);
    $fresh = freshConnection($map, $other, $b);

    app(CleanStaleMapConnectionsAction::class)->handle($map);

    expect(MapConnection::find($stale->id))->toBeNull()
        ->and(MapConnection::find($fresh->id))->not->toBeNull()
        ->and(MapSolarsystem::find($b->id))->not->toBeNull();
});

it('never deletes a pinned system even when it is left unreachable', function () {
    $map = Map::factory()->create();
    $anchor = pinnedSystem($map, 30004001);
    $pinnedLeaf = pinnedSystem($map, 30004002);

    staleConnection($map, $anchor, $pinnedLeaf);

    app(CleanStaleMapConnectionsAction::class)->handle($map);

    expect(MapSolarsystem::find($pinnedLeaf->id))->not->toBeNull();
});

it('never deletes the home system even when it is unreachable from a pinned anchor', function () {
    $map = Map::factory()->create();
    $anchor = pinnedSystem($map, 30004001);
    $home = placeMapSolarsystem($map, 30004002);
    $map->update(['home_solarsystem_id' => $home->solarsystem_id]);

    staleConnection($map, $anchor, $home);

    app(CleanStaleMapConnectionsAction::class)->handle($map);

    expect(MapSolarsystem::find($home->id))->not->toBeNull();
});

it('leaves connections that are not yet stale untouched', function () {
    $map = Map::factory()->create();
    $anchor = pinnedSystem($map, 30004001);
    $b = placeMapSolarsystem($map, 30004002);
    $c = placeMapSolarsystem($map, 30004003);

    $recentCritical = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $anchor->id,
        'to_map_solarsystem_id' => $b->id,
        'lifetime' => LifetimeStatus::Critical,
        'lifetime_updated_at' => now()->subMinutes(5),
    ]);
    $healthy = freshConnection($map, $anchor, $c);

    $count = app(CleanStaleMapConnectionsAction::class)->handle($map);

    expect($count)->toBe(0)
        ->and(MapConnection::find($recentCritical->id))->not->toBeNull()
        ->and(MapConnection::find($healthy->id))->not->toBeNull()
        ->and(MapSolarsystem::find($b->id))->not->toBeNull()
        ->and(MapSolarsystem::find($c->id))->not->toBeNull();
});

it('cleans the map for an authorized editor over http', function () {
    $map = Map::factory()->create();
    $anchor = pinnedSystem($map, 30004001);
    $leaf = placeMapSolarsystem($map, 30004002);
    $connection = staleConnection($map, $anchor, $leaf);

    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))
        ->create();

    actingAs($user)
        ->delete(route('maps.stale-connections.destroy', $map))
        ->assertRedirect();

    expect(MapConnection::find($connection->id))->toBeNull();
});

it('forbids cleaning the map for a viewer', function () {
    $map = Map::factory()->create();
    $anchor = pinnedSystem($map, 30004001);
    $leaf = placeMapSolarsystem($map, 30004002);
    $connection = staleConnection($map, $anchor, $leaf);

    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Viewer])->for($map)))
        ->create();

    actingAs($user)
        ->delete(route('maps.stale-connections.destroy', $map))
        ->assertForbidden();

    expect(MapConnection::find($connection->id))->not->toBeNull();
});
