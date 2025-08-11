<?php

declare(strict_types=1);

use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use Inertia\Testing\AssertableInertia;

use function Pest\Laravel\actingAs;

it('creates a map', function () {
    $map = Map::factory()->make();
    $character = Character::factory()->create();

    actingAs($character->user)
        ->post(route('maps.store'), [
            'name' => $map->name,
        ])
        ->assertRedirect(route('maps.show', $character->mapAccesses()->first()->map));
});

it('shows a form to create a map', function () {
    $character = Character::factory()->create();

    actingAs($character->user)
        ->get(route('maps.create'))
        ->assertOk()
        ->assertInertia(fn (AssertableInertia $assert) => $assert
            ->component('maps/CreateMap')
        );
});

it('shows a list of maps that the user has access to', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($character, 'accessible')->for($map)->create();

    $map_via_corporation = Map::factory()->create();
    MapAccess::factory()->for($character->corporation, 'accessible')->for($map_via_corporation)->create();

    // Another map that the character does not have access to
    Map::factory(5)->create();

    actingAs($character->user)
        ->get(route('home'))
        ->assertOk()
        ->assertInertia(fn (AssertableInertia $assert) => $assert
            ->component('maps/ShowAllMaps')
            ->has('maps', 2)
        );
});
