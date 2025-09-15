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

it('shows the general settings page for a map', function () {

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    actingAs($user);

    visit(route('maps.settings.general.show', $map))
        ->assertSee('General Settings')
        ->assertSee($map->name)
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});

it('shows the access settings page for a map', function () {

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    actingAs($user);

    visit(route('maps.settings.access.show', $map))
        ->assertSee('Access Control')
        ->assertSee($map->name)
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});

it('shows the preferences settings page for a map', function () {

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    actingAs($user);

    visit(route('maps.settings.preferences.show', $map))
        ->assertSee('Preferences')
        ->assertSee($map->name)
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});

it('shows the routing settings page for a map', function () {

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    actingAs($user);

    visit(route('maps.settings.routing.show', $map))
        ->assertSee('Routing Settings')
        ->assertSee($map->name)
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});
