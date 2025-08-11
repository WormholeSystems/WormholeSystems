<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;

use function Pest\Laravel\actingAs;

it('stores a solarsystem if authorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();
    $solarsystem = Solarsystem::query()->firstWhere('name', 'Jita');

    actingAs($character->user)
        ->post(route('map-solarsystems.store'), [
            'map_id' => $map->id,
            'solarsystem_id' => $solarsystem->id,
        ])
        ->assertRedirectBack();
});

it('it does not store a solarsystem if unauthorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();
    $solarsystem = Solarsystem::query()->firstWhere('name', 'Jita');
    $other_character = Character::factory()->create();

    actingAs($other_character->user)
        ->post(route('map-solarsystems.store'), [
            'map_id' => $map->id,
            'solarsystem_id' => $solarsystem->id,
        ])
        ->assertForbidden();
});

it('updates a map solarsystem if authorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();
    $solarsystem = Solarsystem::query()->firstWhere('name', 'Jita');
    $mapSolarsystem = MapSolarsystem::factory()->for($map)->for($solarsystem)->create();

    actingAs($character->user)
        ->put(route('map-solarsystems.update', $mapSolarsystem), [
            'alias' => 'New Alias',
        ])
        ->assertRedirectBack();
});

it('it does not update a map solarsystem if unauthorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();
    $solarsystem = Solarsystem::query()->firstWhere('name', 'Jita');
    $mapSolarsystem = MapSolarsystem::factory()->for($map)->for($solarsystem)->create();
    $other_character = Character::factory()->create();

    actingAs($other_character->user)
        ->put(route('map-solarsystems.update', $mapSolarsystem), [
            'alias' => 'New Alias',
        ])
        ->assertForbidden();
});

it('deletes a map solarsystem if authorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();
    $solarsystem = Solarsystem::query()->firstWhere('name', 'Jita');
    $map_solarsystem = MapSolarsystem::factory()->for($map)->for($solarsystem)->create();

    actingAs($character->user)
        ->delete(route('map-solarsystems.destroy', $map_solarsystem))
        ->assertRedirectBack();
});

it('it does not delete a map solarsystem if unauthorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();
    $solarsystem = Solarsystem::query()->firstWhere('name', 'Jita');
    $map_solarsystem = MapSolarsystem::factory()->for($map)->for($solarsystem)->create();
    $other_character = Character::factory()->create();

    actingAs($other_character->user)
        ->delete(route('map-solarsystems.destroy', $map_solarsystem))
        ->assertForbidden();
});

it('does not update a map solarsystem if the user only has read access', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create([
        'permission' => Permission::Read,
    ]);
    $solarsystem = Solarsystem::query()->firstWhere('name', 'Jita');
    $mapSolarsystem = MapSolarsystem::factory()->for($map)->for($solarsystem)->create();

    actingAs($character->user)
        ->put(route('map-solarsystems.update', $mapSolarsystem), [
            'alias' => 'New Alias',
        ])
        ->assertForbidden();
});

it('does not delete a map solarsystem if the user only has read access', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create([
        'permission' => Permission::Read,
    ]);
    $solarsystem = Solarsystem::query()->firstWhere('name', 'Jita');
    $mapSolarsystem = MapSolarsystem::factory()->for($map)->for($solarsystem)->create();

    actingAs($character->user)
        ->delete(route('map-solarsystems.destroy', $mapSolarsystem))
        ->assertForbidden();
});

it('does not store a solarsystem if the map does not exist', function () {
    $character = Character::factory()->create();
    $solarsystem = Solarsystem::query()->firstWhere('name', 'Jita');

    actingAs($character->user)
        ->post(route('map-solarsystems.store'), [
            'map_id' => 9999, // Non-existent map ID
            'solarsystem_id' => $solarsystem->id,
        ])
        ->assertNotFound();
});
