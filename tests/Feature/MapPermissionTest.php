<?php

/** @noinspection PhpUnhandledExceptionInspection */

use App\Actions\Map\CreateMapAction;
use App\Actions\MapAccess\CreateMapAccessAction;
use App\Models\Character;

test('creator may access map', function () {
    $character = Character::factory()->create();

    $map_action = \Illuminate\Support\Facades\App::make(CreateMapAction::class);

    $map = $map_action->handle($character, [
        'name' => 'Test',
    ]);

    $this->actingAs($character->user)->get(route('maps.show', $map->slug))
        ->assertOk();
});

test('non-creator may not access map', function () {
    $character = Character::factory()->create();

    $other_character = Character::factory()->create();

    $map_action = \Illuminate\Support\Facades\App::make(CreateMapAction::class);

    $map = $map_action->handle($character, [
        'name' => 'Test',
    ]);

    $this->actingAs($other_character->user)->get(route('maps.show', $map->slug))
        ->assertForbidden();
});

test('character may access if access is granted via corporation', function () {
    $corporation = \App\Models\Corporation::factory()->create();
    $character = Character::factory()->for($corporation)->create();
    $other_character = Character::factory()->for($corporation)->create();

    $map_action = \Illuminate\Support\Facades\App::make(CreateMapAction::class);
    $access_action = \Illuminate\Support\Facades\App::make(CreateMapAccessAction::class);

    $map = $map_action->handle($character, [
        'name' => 'Test',
    ]);

    $access_action->handle($map, $corporation);

    $this->actingAs($other_character->user)->get(route('maps.show', $map->slug))
        ->assertOk();
});

test('character may not access if access is not granted via corporation', function () {
    $character = Character::factory()->create();
    $other_character = Character::factory()->create();

    $map_action = \Illuminate\Support\Facades\App::make(CreateMapAction::class);

    $map = $map_action->handle($character, [
        'name' => 'Test',
    ]);

    $this->actingAs($other_character->user)->get(route('maps.show', $map->slug))
        ->assertForbidden();
});

test('character may access if access is granted via alliance', function () {
    $alliance = \App\Models\Alliance::factory()->create();
    $character = Character::factory()->for($alliance)->create();
    $other_character = Character::factory()->for($alliance)->create();

    $map_action = \Illuminate\Support\Facades\App::make(CreateMapAction::class);
    $access_action = \Illuminate\Support\Facades\App::make(CreateMapAccessAction::class);

    $map = $map_action->handle($character, [
        'name' => 'Test',
    ]);

    $access_action->handle($map, $alliance);

    $this->actingAs($other_character->user)->get(route('maps.show', $map->slug))
        ->assertOk();
});

test('character may not access if access is not granted via alliance', function () {
    $character = Character::factory()->create();
    $other_character = Character::factory()->create();

    $map_action = \Illuminate\Support\Facades\App::make(CreateMapAction::class);

    $map = $map_action->handle($character, [
        'name' => 'Test',
    ]);

    $this->actingAs($other_character->user)->get(route('maps.show', $map->slug))
        ->assertForbidden();
});
