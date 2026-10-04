<?php

declare(strict_types=1);

use App\Enums\ConnectionType;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\User;

use function Pest\Laravel\actingAs;

function connectionUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

function makeConnection(Map $map, array $attributes = []): MapConnection
{
    $from = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => makeSolarsystem(30009601)]);
    $to = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => makeSolarsystem(30009602)]);

    return MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        ...$attributes,
    ]);
}

it('defaults new connections to a wormhole with mass not preserved', function () {
    $connection = makeConnection(Map::factory()->create())->fresh();

    expect($connection->type)->toBe(ConnectionType::Wormhole)
        ->and($connection->preserve_mass)->toBeFalse();
});

it('lets a member mark a connection as a stargate', function () {
    $map = Map::factory()->create();
    $connection = makeConnection($map);

    actingAs(connectionUser($map, Permission::Member))
        ->put("/map-connections/{$connection->id}", ['type' => 'stargate'])
        ->assertRedirect();

    expect($connection->fresh()->type)->toBe(ConnectionType::Stargate);
});

it('lets a member toggle preserve mass on a connection', function () {
    $map = Map::factory()->create();
    $connection = makeConnection($map, ['preserve_mass' => false]);

    actingAs(connectionUser($map, Permission::Member))
        ->put("/map-connections/{$connection->id}", ['preserve_mass' => true])
        ->assertRedirect();

    expect($connection->fresh()->preserve_mass)->toBeTrue();
});

it('rejects an invalid connection type', function () {
    $map = Map::factory()->create();
    $connection = makeConnection($map);

    actingAs(connectionUser($map, Permission::Member))
        ->put("/map-connections/{$connection->id}", ['type' => 'bridge'])
        ->assertSessionHasErrors('type');

    expect($connection->fresh()->type)->toBe(ConnectionType::Wormhole);
});

it('forbids a viewer from updating a connection', function () {
    $map = Map::factory()->create();
    $connection = makeConnection($map);

    actingAs(connectionUser($map, Permission::Viewer))
        ->put("/map-connections/{$connection->id}", ['type' => 'stargate'])
        ->assertForbidden();

    expect($connection->fresh()->type)->toBe(ConnectionType::Wormhole);
});

/**
 * Manually draw a connection between two placed systems and return its stored type.
 */
function storeManualConnection(int $from_solarsystem_id, int $to_solarsystem_id, array $attributes = []): ConnectionType
{
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, $from_solarsystem_id);
    $to = placeMapSolarsystem($map, $to_solarsystem_id, 300, 300);

    actingAs(connectionUser($map, Permission::Member))
        ->post('/map-connections', [
            'from_map_solarsystem_id' => $from->id,
            'to_map_solarsystem_id' => $to->id,
            ...$attributes,
        ])
        ->assertRedirect();

    return MapConnection::query()->where('map_id', $map->id)->sole()->type;
}

it('creates a manual connection between gate neighbors as a stargate', function () {
    makeSolarsystem(30009611, type: 'eve');
    makeSolarsystem(30009612, type: 'eve');
    linkSolarsystemsByStargate(30009611, 30009612);

    expect(storeManualConnection(30009611, 30009612))->toBe(ConnectionType::Stargate);
});

it('creates a manual connection between k-space systems without a gate as a wormhole', function () {
    makeSolarsystem(30009613, type: 'eve');
    makeSolarsystem(30009614, type: 'eve');

    expect(storeManualConnection(30009613, 30009614))->toBe(ConnectionType::Wormhole);
});

it('creates a manual connection involving wormhole space as a wormhole', function () {
    makeSolarsystem(31009615, type: 'wormhole');
    makeSolarsystem(31009616, type: 'wormhole');
    linkSolarsystemsByStargate(31009615, 31009616);

    expect(storeManualConnection(31009615, 31009616))->toBe(ConnectionType::Wormhole);
});

it('keeps a manual connection with an identified wormhole type as a wormhole', function () {
    makeSolarsystem(30009617, type: 'eve');
    makeSolarsystem(30009618, type: 'eve');
    linkSolarsystemsByStargate(30009617, 30009618);

    expect(storeManualConnection(30009617, 30009618, ['wormhole_id' => makeWormhole()->id]))->toBe(ConnectionType::Wormhole);
});
