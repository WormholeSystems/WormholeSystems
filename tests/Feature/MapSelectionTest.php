<?php

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapSolarsystem;
use Illuminate\Database\Eloquent\Collection;

use function Pest\Laravel\actingAs;

it('updates the map selected if authorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();

    /** @var Collection<int, MapSolarsystem> $mapSolarsystems */
    $mapSolarsystems = MapSolarsystem::factory()->for($map)->count(5)->create();

    $data = $mapSolarsystems->map(
        fn (MapSolarsystem $mapSolarsystem) => [
            'id' => $mapSolarsystem->id,
            'position_x' => $mapSolarsystem->position_x,
            'position_y' => $mapSolarsystem->position_y,
        ]
    );

    actingAs($character->user)
        ->put(route('map-selection.update'), [
            'map_solarsystems' => $data->values()->all(),
        ])
        ->assertRedirectBack()
        ->assertSessionHasNoErrors();
});

it('does not update the map selected if unauthorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();

    /** @var Collection<int, MapSolarsystem> $mapSolarsystems */
    $mapSolarsystems = MapSolarsystem::factory(5)->for($map)->create();

    $data = $mapSolarsystems->map(
        fn (MapSolarsystem $mapSolarsystem) => [
            'id' => $mapSolarsystem->id,
            'position_x' => $mapSolarsystem->position_x,
            'position_y' => $mapSolarsystem->position_y,
        ]
    );

    actingAs($character->user)
        ->put(route('map-selection.update'), [
            'map_solarsystems' => $data->all(),
        ])
        ->assertForbidden();
});

it('does not update the map selection if the user only has read access', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create([
        'permission' => Permission::Read,
    ]);

    /** @var Collection<int, MapSolarsystem> $mapSolarsystems */
    $mapSolarsystems = MapSolarsystem::factory(5)->for($map)->create();

    $data = $mapSolarsystems->map(
        fn (MapSolarsystem $mapSolarsystem) => [
            'id' => $mapSolarsystem->id,
            'position_x' => $mapSolarsystem->position_x,
            'position_y' => $mapSolarsystem->position_y,
        ]
    );

    actingAs($character->user)
        ->put(route('map-selection.update'), [
            'map_solarsystems' => $data->all(),
        ])
        ->assertForbidden();
});
