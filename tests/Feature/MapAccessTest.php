<?php

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;

use function Pest\Laravel\actingAs;

it('updates the map access if authorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();

    $another_character = Character::factory()->create();

    actingAs($character->user)
        ->post(route('map-access.store', $map), [
            'entity_id' => $another_character->id,
            'entity_type' => 'character',
            'permission' => Permission::Write->value,
        ])
        ->assertRedirectBack()
        ->assertSessionHasNoErrors();
});

it('does not update the map access if unauthorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create([
        'permission' => Permission::Read->value,
    ]);
    $another_character = Character::factory()->create();

    actingAs($another_character->user)
        ->post(route('map-access.store', $map), [
            'entity_id' => $character->id,
            'entity_type' => 'character',
            'permission' => Permission::Write->value,
        ])
        ->assertForbidden();
});

it('does not update the map access if the user is the owner of the map', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create([
        'permission' => Permission::Write->value,
    ]);

    $another_character = Character::factory()->create();
    MapAccess::factory()->for($map)->for($another_character, 'accessible')->create([
        'permission' => Permission::Write->value,
        'is_owner' => true,
    ]);

    actingAs($character->user)
        ->post(route('map-access.store', $map), [
            'entity_id' => $another_character->id,
            'entity_type' => 'character',
            'permission' => Permission::Write->value,
        ])
        ->assertForbidden();
});
