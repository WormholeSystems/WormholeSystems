<?php

declare(strict_types=1);

use App\Models\Map;
use App\Models\User;

use function Pest\Laravel\actingAs;

it('shows all the maps the user has access to', function () {

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    $other_map = Map::factory()->create();

    actingAs($user);

    visit(route('home'))
        ->assertSee($map->name)
        ->assertDontSee($other_map->name)
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});

it('shows a map', function () {

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    actingAs($user);

    visit(route('maps.show', $map))
        ->assertSee('Welcome to the Map')
        ->press('Next')
        ->assertSee('Grant permissions')
        ->press('Skip for now')
        ->assertSee('Configure Settings')
        ->press('Next')
        ->assertSee('Ready to Explore')
        ->press('Start Exploring!')
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});
