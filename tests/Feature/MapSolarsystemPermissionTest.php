<?php

/** @noinspection PhpUnhandledExceptionInspection */

use App\Actions\Map\CreateMapAction;
use App\Models\Character;
use App\Models\Solarsystem;

test('creator can add a map solarsystem', function () {
    $character = Character::factory()->create();

    $action = $this->app->make(CreateMapAction::class);

    $map = $action->handle(
        $character,
        [
            'name' => 'Test Map',
        ]
    );

    $solarsystem = Solarsystem::firstWhere('name', 'Jita');

    $this->actingAs($character->user)
        ->post(route('map-solarsystems.store'), [
            'map_id' => $map->id,
            'solarsystem_id' => $solarsystem->id,
        ])
        ->assertRedirectBack();
});

test('non-creator cannot add a map solarsystem', function () {
    $character = Character::factory()->create();

    $action = $this->app->make(CreateMapAction::class);

    $map = $action->handle(
        $character,
        [
            'name' => 'Test Map',
        ]
    );

    $solarsystem = Solarsystem::firstWhere('name', 'Jita');

    $this->actingAs(Character::factory()->create()->user)
        ->post(route('map-solarsystems.store'), [
            'map_id' => $map->id,
            'solarsystem_id' => $solarsystem->id,
        ])
        ->assertForbidden();
});

test('creator can update a map solarsystem', function () {
    $character = Character::factory()->create();

    $action = $this->app->make(CreateMapAction::class);

    $map = $action->handle(
        $character,
        [
            'name' => 'Test Map',
        ]
    );

    $solarsystem = Solarsystem::firstWhere('name', 'Jita');

    $this->actingAs($character->user)
        ->post(route('map-solarsystems.store'), [
            'map_id' => $map->id,
            'solarsystem_id' => $solarsystem->id,
        ])
        ->assertRedirectBack();

    $mapSolarsystem = $map->mapSolarsystems()->first();

    $this->actingAs($character->user)
        ->put(route('map-solarsystems.update', $mapSolarsystem), [
            'alias' => 'New Alias',
        ])
        ->assertRedirectBack();
});

test('non-creator cannot update a map solarsystem', function () {
    $character = Character::factory()->create();

    $action = $this->app->make(CreateMapAction::class);

    $map = $action->handle(
        $character,
        [
            'name' => 'Test Map',
        ]
    );

    $solarsystem = Solarsystem::firstWhere('name', 'Jita');

    $this->actingAs($character->user)
        ->post(route('map-solarsystems.store'), [
            'map_id' => $map->id,
            'solarsystem_id' => $solarsystem->id,
        ])
        ->assertRedirectBack();

    $mapSolarsystem = $map->mapSolarsystems()->first();

    $this->actingAs(Character::factory()->create()->user)
        ->put(route('map-solarsystems.update', $mapSolarsystem), [
            'alias' => 'New Alias',
        ])
        ->assertForbidden();
});

test('creator can delete a map solarsystem', function () {
    $character = Character::factory()->create();

    $action = $this->app->make(CreateMapAction::class);

    $map = $action->handle(
        $character,
        [
            'name' => 'Test Map',
        ]
    );

    $solarsystem = Solarsystem::firstWhere('name', 'Jita');

    $this->actingAs($character->user)
        ->post(route('map-solarsystems.store'), [
            'map_id' => $map->id,
            'solarsystem_id' => $solarsystem->id,
        ])
        ->assertRedirectBack();

    $mapSolarsystem = $map->mapSolarsystems()->first();

    $this->actingAs($character->user)
        ->delete(route('map-solarsystems.destroy', $mapSolarsystem))
        ->assertRedirectBack();
});

test('non-creator cannot delete a map solarsystem', function () {
    $character = Character::factory()->create();

    $action = $this->app->make(CreateMapAction::class);

    $map = $action->handle(
        $character,
        [
            'name' => 'Test Map',
        ]
    );

    $solarsystem = Solarsystem::firstWhere('name', 'Jita');

    $this->actingAs($character->user)
        ->post(route('map-solarsystems.store'), [
            'map_id' => $map->id,
            'solarsystem_id' => $solarsystem->id,
        ])
        ->assertRedirectBack();

    $mapSolarsystem = $map->mapSolarsystems()->first();

    $this->actingAs(Character::factory()->create()->user)
        ->delete(route('map-solarsystems.destroy', $mapSolarsystem))
        ->assertForbidden();
});
