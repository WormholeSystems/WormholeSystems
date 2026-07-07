<?php

declare(strict_types=1);

use App\Models\Map;
use App\Models\MapSolarsystemDetails;
use App\Models\User;
use App\Models\WormholeSystem;

use function Pest\Laravel\actingAs;

function createThreatSystem(int $solarsystemId, array $threatData): WormholeSystem
{
    $wormholeSystem = WormholeSystem::query()->create(['id' => $solarsystemId]);

    $wormholeSystem->threats()->createMany(array_map(
        fn (array $entity): array => [
            'entity_id' => $entity['id'],
            'entity_type' => $entity['type'],
            'name' => $entity['name'],
            'kills' => $entity['kills'],
        ],
        $threatData,
    ));

    return $wormholeSystem;
}

it('finds systems on the map by threat entity name', function () {
    $map = Map::factory()->create();
    $tuskersHome = placeMapSolarsystem($map, 31000101);
    $tuskersSecond = placeMapSolarsystem($map, 31000102);
    placeMapSolarsystem($map, 31000103);

    createThreatSystem(31000101, [
        ['id' => 98505600, 'name' => 'The Tuskers', 'type' => 'corporation', 'kills' => 12],
        ['id' => 99000001, 'name' => 'Goonswarm Federation', 'type' => 'alliance', 'kills' => 40],
    ]);
    createThreatSystem(31000102, [
        ['id' => 98505600, 'name' => 'The Tuskers', 'type' => 'corporation', 'kills' => 30],
    ]);
    createThreatSystem(31000103, [
        ['id' => 99000001, 'name' => 'Goonswarm Federation', 'type' => 'alliance', 'kills' => 5],
    ]);

    $user = User::factory()->ownsMap($map)->create();
    actingAs($user);

    $response = $this->getJson(route('maps.search', ['map' => $map, 'q' => 'tusk']))
        ->assertSuccessful()
        ->json('threats');

    expect($response)->toHaveCount(1)
        ->and($response[0]['name'])->toBe('The Tuskers')
        ->and($response[0]['type'])->toBe('corporation')
        ->and($response[0]['total_kills'])->toBe(42)
        ->and($response[0]['systems'])->toEqual([
            ['solarsystem_id' => 31000102, 'kills' => 30, 'occupier_alias' => $tuskersSecond->details->occupier_alias],
            ['solarsystem_id' => 31000101, 'kills' => 12, 'occupier_alias' => $tuskersHome->details->occupier_alias],
        ]);

    expect($tuskersHome->solarsystem_id)->toBe(31000101)
        ->and($tuskersSecond->solarsystem_id)->toBe(31000102);
});

it('includes systems that are not on the map with their remembered occupier alias', function () {
    $map = Map::factory()->create();
    placeMapSolarsystem($map, 31000111);
    makeSolarsystem(31000112);
    MapSolarsystemDetails::factory()->create([
        'map_id' => $map->id,
        'solarsystem_id' => 31000112,
        'occupier_alias' => 'Hard Knocks HQ',
    ]);

    createThreatSystem(31000111, [['id' => 1, 'name' => 'Hard Knocks Inc.', 'type' => 'corporation', 'kills' => 3]]);
    createThreatSystem(31000112, [['id' => 1, 'name' => 'Hard Knocks Inc.', 'type' => 'corporation', 'kills' => 99]]);

    $user = User::factory()->ownsMap($map)->create();
    actingAs($user);

    $response = $this->getJson(route('maps.search', ['map' => $map, 'q' => 'hard knocks']))
        ->assertSuccessful()
        ->json('threats');

    expect($response)->toHaveCount(1)
        ->and($response[0]['total_kills'])->toBe(102)
        ->and($response[0]['systems_count'])->toBe(2)
        ->and($response[0]['systems'][0])->toEqual(['solarsystem_id' => 31000112, 'kills' => 99, 'occupier_alias' => 'Hard Knocks HQ']);
});

it('caps the per-entity system list at ten and reports the full count', function () {
    $map = Map::factory()->create();

    foreach (range(1, 12) as $index) {
        makeSolarsystem(31000200 + $index);
        createThreatSystem(31000200 + $index, [['id' => 7, 'name' => 'Lazerhawks', 'type' => 'alliance', 'kills' => $index]]);
    }

    $user = User::factory()->ownsMap($map)->create();
    actingAs($user);

    $response = $this->getJson(route('maps.search', ['map' => $map, 'q' => 'lazerhawks']))
        ->assertSuccessful()
        ->json('threats');

    expect($response[0]['systems_count'])->toBe(12)
        ->and($response[0]['systems'])->toHaveCount(10)
        ->and($response[0]['systems'][0]['kills'])->toBe(12);
});

it('sorts entities by total kills and matches case-insensitively', function () {
    $map = Map::factory()->create();
    placeMapSolarsystem($map, 31000121);

    createThreatSystem(31000121, [
        ['id' => 1, 'name' => 'Lazerhawks', 'type' => 'alliance', 'kills' => 10],
        ['id' => 2, 'name' => 'LAZERS EVERYWHERE', 'type' => 'corporation', 'kills' => 25],
    ]);

    $user = User::factory()->ownsMap($map)->create();
    actingAs($user);

    $response = $this->getJson(route('maps.search', ['map' => $map, 'q' => 'LaZer']))
        ->assertSuccessful()
        ->json('threats');

    expect($response)->toHaveCount(2)
        ->and($response[0]['name'])->toBe('LAZERS EVERYWHERE')
        ->and($response[1]['name'])->toBe('Lazerhawks');
});

it('rejects queries shorter than two characters', function () {
    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();
    actingAs($user);

    $this->getJson(route('maps.search', ['map' => $map, 'q' => 'a']))
        ->assertUnprocessable();
});

it('forbids users without access to the map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->create();
    actingAs($user);

    $this->getJson(route('maps.search', ['map' => $map, 'q' => 'tuskers']))
        ->assertForbidden();
});
