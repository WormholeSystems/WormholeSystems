<?php

use App\Enums\MassStatus;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;

use function Pest\Laravel\actingAs;

it('stores a map connection if authorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();
    $solarsystem1 = Solarsystem::query()->firstWhere('name', 'Jita');
    $solarsystem2 = Solarsystem::query()->firstWhere('name', 'Amarr');
    $map_solarsystem1 = MapSolarsystem::factory()->for($map)->for($solarsystem1)->create();
    $map_solarsystem2 = MapSolarsystem::factory()->for($map)->for($solarsystem2)->create();

    actingAs($character->user)
        ->post(route('map-connections.store'), [
            'from_map_solarsystem_id' => $map_solarsystem1->id,
            'to_map_solarsystem_id' => $map_solarsystem2->id,
        ])
        ->assertRedirectBack()
        ->assertSessionHasNoErrors();
});

test('it does not store a map connection if unauthorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    $solarsystem1 = Solarsystem::query()->firstWhere('name', 'Jita');
    $solarsystem2 = Solarsystem::query()->firstWhere('name', 'Amarr');
    $map_solarsystem1 = MapSolarsystem::factory()->for($map)->for($solarsystem1)->create();
    $map_solarsystem2 = MapSolarsystem::factory()->for($map)->for($solarsystem2)->create();

    actingAs($character->user)
        ->post(route('map-connections.store'), [
            'from_map_solarsystem_id' => $map_solarsystem1->id,
            'to_map_solarsystem_id' => $map_solarsystem2->id,
        ])
        ->assertForbidden();
});

it('updates a map connection if authorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();
    $solarsystem1 = Solarsystem::query()->firstWhere('name', 'Jita');
    $solarsystem2 = Solarsystem::query()->firstWhere('name', 'Amarr');
    $map_solarsystem1 = MapSolarsystem::factory()->for($map)->for($solarsystem1)->create();
    $map_solarsystem2 = MapSolarsystem::factory()->for($map)->for($solarsystem2)->create();
    $connection = MapConnection::factory()->for($map)->for($map_solarsystem1, 'fromMapSolarsystem')->for($map_solarsystem2, 'toMapSolarsystem')->create();

    actingAs($character->user)
        ->put(route('map-connections.update', $connection), [
            'mass_status' => MassStatus::Critical->value,
        ])
        ->assertRedirectBack()
        ->assertSessionHasNoErrors();
});

test('it does not update a map connection if unauthorized', function () {
    $map = Map::factory()->create();
    $solarsystem1 = Solarsystem::query()->firstWhere('name', 'Jita');
    $solarsystem2 = Solarsystem::query()->firstWhere('name', 'Amarr');
    $map_solarsystem1 = MapSolarsystem::factory()->for($map)->for($solarsystem1)->create();
    $map_solarsystem2 = MapSolarsystem::factory()->for($map)->for($solarsystem2)->create();
    $connection = MapConnection::factory()->for($map)->for($map_solarsystem1, 'fromMapSolarsystem')->for($map_solarsystem2, 'toMapSolarsystem')->create();
    $character = Character::factory()->create();

    actingAs($character->user)
        ->put(route('map-connections.update', $connection), [
            'mass_status' => MassStatus::Critical->value,
        ])
        ->assertForbidden();
});

it('deletes a map connection if authorized', function () {
    $character = Character::factory()->create();
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->for($character, 'accessible')->create();
    $solarsystem1 = Solarsystem::query()->firstWhere('name', 'Jita');
    $solarsystem2 = Solarsystem::query()->firstWhere('name', 'Amarr');
    $map_solarsystem1 = MapSolarsystem::factory()->for($map)->for($solarsystem1)->create();
    $map_solarsystem2 = MapSolarsystem::factory()->for($map)->for($solarsystem2)->create();
    $connection = MapConnection::factory()->for($map)->for($map_solarsystem1, 'fromMapSolarsystem')->for($map_solarsystem2, 'toMapSolarsystem')->create();

    actingAs($character->user)
        ->delete(route('map-connections.destroy', $connection))
        ->assertRedirectBack()
        ->assertSessionHasNoErrors();
});

it('does not delete a map connection if unauthorized', function () {
    $map = Map::factory()->create();
    $solarsystem1 = Solarsystem::query()->firstWhere('name', 'Jita');
    $solarsystem2 = Solarsystem::query()->firstWhere('name', 'Amarr');
    $map_solarsystem1 = MapSolarsystem::factory()->for($map)->for($solarsystem1)->create();
    $map_solarsystem2 = MapSolarsystem::factory()->for($map)->for($solarsystem2)->create();
    $connection = MapConnection::factory()->for($map)->for($map_solarsystem1, 'fromMapSolarsystem')->for($map_solarsystem2, 'toMapSolarsystem')->create();
    $character = Character::factory()->create();

    actingAs($character->user)
        ->delete(route('map-connections.destroy', $connection))
        ->assertForbidden();
});
