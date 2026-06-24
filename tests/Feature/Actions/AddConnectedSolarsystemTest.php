<?php

declare(strict_types=1);

use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\User;

use function Pest\Laravel\actingAs;

function memberOfMap(Map $map): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

function connect(Map $map, int $from, int $to): MapConnection
{
    return MapConnection::create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from,
        'to_map_solarsystem_id' => $to,
    ]);
}

it('connects the new system to the origin even when the origin already has connections', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30013001);

    // A pre-existing link pointing AT the origin: the dedup guard must not treat this as
    // "already connected" to the new target (the bug it once did via an OR mismatch).
    $neighbour = placeMapSolarsystem($map, 30013002);
    connect($map, $neighbour->id, $origin->id);

    $targetId = makeSolarsystem(30013003);

    $target = app(StoreMapSolarsystemAction::class)->handle($map, [
        'solarsystem_id' => $targetId,
        'position_x' => 100,
        'position_y' => 100,
        'connect_to_map_solarsystem_id' => $origin->id,
    ]);

    expect(MapConnection::query()
        ->where('map_id', $map->id)
        ->where('from_map_solarsystem_id', $origin->id)
        ->where('to_map_solarsystem_id', $target->id)
        ->exists())->toBeTrue();
});

it('does not duplicate the connection when the two are already linked', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30013004);
    $existing = placeMapSolarsystem($map, 30013005);
    connect($map, $origin->id, $existing->id);

    app(StoreMapSolarsystemAction::class)->handle($map, [
        'solarsystem_id' => $existing->solarsystem_id,
        'position_x' => 100,
        'position_y' => 100,
        'connect_to_map_solarsystem_id' => $origin->id,
    ]);

    expect(MapConnection::where('map_id', $map->id)->count())->toBe(1);
});

it('links to a system already on the map without moving it', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30013010);
    $existing = placeMapSolarsystem($map, 30013011, 720, 480);

    app(StoreMapSolarsystemAction::class)->handle($map, [
        'solarsystem_id' => $existing->solarsystem_id,
        // A position that would relocate the system if it were applied.
        'position_x' => 100,
        'position_y' => 100,
        'connect_to_map_solarsystem_id' => $origin->id,
    ]);

    $existing->refresh();

    expect($existing->position_x)->toEqual(720)
        ->and($existing->position_y)->toEqual(480)
        ->and(MapConnection::query()
            ->where('map_id', $map->id)
            ->where('from_map_solarsystem_id', $origin->id)
            ->where('to_map_solarsystem_id', $existing->id)
            ->exists())->toBeTrue();
});

it('adds a connected system through the HTTP endpoint', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30013006);
    $neighbour = placeMapSolarsystem($map, 30013007);
    connect($map, $neighbour->id, $origin->id);

    $targetId = makeSolarsystem(30013008);

    actingAs(memberOfMap($map))
        ->post('/map-solarsystems', [
            'map_id' => $map->id,
            'solarsystem_id' => $targetId,
            'position_x' => 100,
            'position_y' => 100,
            'connect_to_map_solarsystem_id' => $origin->id,
        ])
        ->assertRedirect();

    $target = $map->mapSolarsystems()->where('solarsystem_id', $targetId)->first();

    expect($target)->not->toBeNull()
        ->and(MapConnection::query()
            ->where('map_id', $map->id)
            ->where('from_map_solarsystem_id', $origin->id)
            ->where('to_map_solarsystem_id', $target->id)
            ->exists())->toBeTrue();
});
