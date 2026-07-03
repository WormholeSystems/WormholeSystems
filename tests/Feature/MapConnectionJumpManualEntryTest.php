<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Events\MapConnections\MapConnectionsUpsertedEvent;
use App\Models\Category;
use App\Models\Character;
use App\Models\Group;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\MapConnectionJump;
use App\Models\Type;
use App\Models\User;
use Illuminate\Support\Facades\Event;

use function Pest\Laravel\actingAs;

const MANUAL_JUMP_SHIP_TYPE_ID = 73791;

const MANUAL_JUMP_SHIP_MASS = 130_000_000;

function manualJumpUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

/**
 * @return array{connection: MapConnection, from: int, to: int}
 */
function manualJumpConnection(Map $map): array
{
    $from = placeMapSolarsystem($map, 31009601);
    $to = placeMapSolarsystem($map, 31009602, 300, 300);

    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
    ]);

    return ['connection' => $connection, 'from' => 31009601, 'to' => 31009602];
}

function manualJumpShipType(): Type
{
    Category::query()->firstOrCreate(['id' => 6], ['name' => 'Ship']);
    Group::query()->firstOrCreate(['id' => 27], ['name' => 'Battleship', 'category_id' => 6]);

    return Type::query()->firstOrCreate(
        ['id' => MANUAL_JUMP_SHIP_TYPE_ID],
        ['name' => 'Megathron', 'group_id' => 27, 'mass' => MANUAL_JUMP_SHIP_MASS],
    );
}

it('lets a member log a manual jump with the mass derived from the ship type', function () {
    Event::fake([MapConnectionsUpsertedEvent::class]);

    $map = Map::factory()->create();
    ['connection' => $connection, 'from' => $from, 'to' => $to] = manualJumpConnection($map);
    manualJumpShipType();

    actingAs(manualJumpUser($map, Permission::Member))
        ->post('/map-connection-jumps', [
            'map_connection_id' => $connection->id,
            'direction' => 'outbound',
            'ship_type_id' => MANUAL_JUMP_SHIP_TYPE_ID,
        ])
        ->assertRedirect();

    $jump = MapConnectionJump::query()->sole();
    expect($jump->map_connection_id)->toBe($connection->id)
        ->and($jump->map_id)->toBe($map->id)
        ->and($jump->character_id)->toBeNull()
        ->and($jump->is_manual)->toBeTrue()
        ->and($jump->mass)->toBe(MANUAL_JUMP_SHIP_MASS)
        ->and($jump->from_solarsystem_id)->toBe($from)
        ->and($jump->to_solarsystem_id)->toBe($to);

    Event::assertDispatched(MapConnectionsUpsertedEvent::class);
});

it('prefers an explicit mass over the ship type mass and honors the inbound direction', function () {
    $map = Map::factory()->create();
    ['connection' => $connection, 'from' => $from, 'to' => $to] = manualJumpConnection($map);
    manualJumpShipType();

    actingAs(manualJumpUser($map, Permission::Member))
        ->post('/map-connection-jumps', [
            'map_connection_id' => $connection->id,
            'direction' => 'inbound',
            'ship_type_id' => MANUAL_JUMP_SHIP_TYPE_ID,
            'mass' => 300_000_000,
        ])
        ->assertRedirect();

    $jump = MapConnectionJump::query()->sole();
    expect($jump->mass)->toBe(300_000_000)
        ->and($jump->from_solarsystem_id)->toBe($to)
        ->and($jump->to_solarsystem_id)->toBe($from);
});

it('rejects a manual jump without a mass or ship type', function () {
    $map = Map::factory()->create();
    ['connection' => $connection] = manualJumpConnection($map);

    actingAs(manualJumpUser($map, Permission::Member))
        ->post('/map-connection-jumps', [
            'map_connection_id' => $connection->id,
            'direction' => 'outbound',
        ])
        ->assertSessionHasErrors('mass');

    expect(MapConnectionJump::query()->count())->toBe(0);
});

it('lets a member correct the mass of a tracked jump', function () {
    Event::fake([MapConnectionsUpsertedEvent::class]);

    $map = Map::factory()->create();
    ['connection' => $connection, 'from' => $from, 'to' => $to] = manualJumpConnection($map);
    $user = manualJumpUser($map, Permission::Member);

    $jump = MapConnectionJump::factory()->create([
        'map_id' => $map->id,
        'map_connection_id' => $connection->id,
        'character_id' => $user->characters()->value('id'),
        'from_solarsystem_id' => $from,
        'to_solarsystem_id' => $to,
        'mass' => 100_000_000,
    ]);

    actingAs($user)
        ->put("/map-connection-jumps/{$jump->id}", ['mass' => 250_000_000])
        ->assertRedirect();

    expect($jump->fresh()->mass)->toBe(250_000_000)
        ->and($jump->fresh()->is_manual)->toBeFalse();

    Event::assertDispatched(MapConnectionsUpsertedEvent::class);
});

it('re-derives the mass when the ship type changes without an explicit mass', function () {
    $map = Map::factory()->create();
    ['connection' => $connection, 'from' => $from, 'to' => $to] = manualJumpConnection($map);
    manualJumpShipType();
    $user = manualJumpUser($map, Permission::Member);

    $jump = MapConnectionJump::factory()->create([
        'map_id' => $map->id,
        'map_connection_id' => $connection->id,
        'character_id' => $user->characters()->value('id'),
        'from_solarsystem_id' => $from,
        'to_solarsystem_id' => $to,
        'mass' => 100_000_000,
    ]);

    actingAs($user)
        ->put("/map-connection-jumps/{$jump->id}", ['ship_type_id' => MANUAL_JUMP_SHIP_TYPE_ID, 'direction' => 'inbound'])
        ->assertRedirect();

    $fresh = $jump->fresh();
    expect($fresh->mass)->toBe(MANUAL_JUMP_SHIP_MASS)
        ->and($fresh->ship_type_id)->toBe(MANUAL_JUMP_SHIP_TYPE_ID)
        ->and($fresh->from_solarsystem_id)->toBe($to)
        ->and($fresh->to_solarsystem_id)->toBe($from);
});

it('lets a member delete a jump entry', function () {
    Event::fake([MapConnectionsUpsertedEvent::class]);

    $map = Map::factory()->create();
    ['connection' => $connection, 'from' => $from, 'to' => $to] = manualJumpConnection($map);
    $user = manualJumpUser($map, Permission::Member);

    $jump = MapConnectionJump::factory()->create([
        'map_id' => $map->id,
        'map_connection_id' => $connection->id,
        'character_id' => $user->characters()->value('id'),
        'from_solarsystem_id' => $from,
        'to_solarsystem_id' => $to,
    ]);

    actingAs($user)
        ->delete("/map-connection-jumps/{$jump->id}")
        ->assertRedirect();

    expect(MapConnectionJump::query()->count())->toBe(0);

    Event::assertDispatched(MapConnectionsUpsertedEvent::class);
});

it('forbids a viewer from logging, editing or deleting jumps', function () {
    $map = Map::factory()->create();
    ['connection' => $connection, 'from' => $from, 'to' => $to] = manualJumpConnection($map);
    $viewer = manualJumpUser($map, Permission::Viewer);

    $jump = MapConnectionJump::factory()->create([
        'map_id' => $map->id,
        'map_connection_id' => $connection->id,
        'character_id' => null,
        'from_solarsystem_id' => $from,
        'to_solarsystem_id' => $to,
        'is_manual' => true,
    ]);

    actingAs($viewer)
        ->post('/map-connection-jumps', [
            'map_connection_id' => $connection->id,
            'direction' => 'outbound',
            'mass' => 100_000_000,
        ])
        ->assertForbidden();

    actingAs($viewer)
        ->put("/map-connection-jumps/{$jump->id}", ['mass' => 1])
        ->assertForbidden();

    actingAs($viewer)
        ->delete("/map-connection-jumps/{$jump->id}")
        ->assertForbidden();

    expect(MapConnectionJump::query()->sole()->mass)->not->toBe(1);
});
