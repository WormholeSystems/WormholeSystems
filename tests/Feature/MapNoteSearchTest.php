<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapSolarsystemDetails;
use App\Models\User;

use function Pest\Laravel\actingAs;

function noteSearchUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

it('finds systems by note text and returns an excerpt around the match', function () {
    $map = Map::factory()->create();
    $withNote = placeMapSolarsystem($map, 31000301);
    $withoutMatch = placeMapSolarsystem($map, 31000302);

    $withNote->details->update(['notes' => "# Staging\n\nDread bomb staged here, contact Fabian before engaging anything."]);
    $withoutMatch->details->update(['notes' => 'Nothing to see.']);

    actingAs(noteSearchUser($map, Permission::Member));

    $response = $this->getJson(route('maps.search', ['map' => $map, 'q' => 'dread bomb']))
        ->assertSuccessful()
        ->json('notes');

    expect($response)->toHaveCount(1)
        ->and($response[0]['map_solarsystem_id'])->toBe($withNote->id)
        ->and($response[0]['solarsystem_id'])->toBe(31000301)
        ->and($response[0]['note_excerpt'])->toContain('Dread bomb staged here');
});

it('returns nothing for viewers because notes are hidden from them', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 31000303);
    $system->details->update(['notes' => 'Secret staging intel.']);

    actingAs(noteSearchUser($map, Permission::Viewer));

    $this->getJson(route('maps.search', ['map' => $map, 'q' => 'staging']))
        ->assertSuccessful()
        ->assertExactJson(['threats' => [], 'notes' => [], 'occupiers' => []]);
});

it('does not leak notes from other maps', function () {
    $map = Map::factory()->create();
    $otherMap = Map::factory()->create();
    $otherSystem = placeMapSolarsystem($otherMap, 31000304);
    $otherSystem->details->update(['notes' => 'Other map staging notes.']);

    actingAs(noteSearchUser($map, Permission::Member));

    $this->getJson(route('maps.search', ['map' => $map, 'q' => 'staging']))
        ->assertSuccessful()
        ->assertExactJson(['threats' => [], 'notes' => [], 'occupiers' => []]);
});

it('forbids users without access to the map', function () {
    $map = Map::factory()->create();
    actingAs(User::factory()->create());

    $this->getJson(route('maps.search', ['map' => $map, 'q' => 'staging']))
        ->assertForbidden();
});

it('finds systems by occupier alias even when they are no longer on the map', function () {
    $map = Map::factory()->create();
    $otherMap = Map::factory()->create();
    makeSolarsystem(31000305);
    makeSolarsystem(31000306);

    MapSolarsystemDetails::factory()->create([
        'map_id' => $map->id,
        'solarsystem_id' => 31000305,
        'occupier_alias' => 'Tuskers Home',
    ]);
    MapSolarsystemDetails::factory()->create([
        'map_id' => $otherMap->id,
        'solarsystem_id' => 31000306,
        'occupier_alias' => 'Tuskers Forward Base',
    ]);

    actingAs(noteSearchUser($map, Permission::Member));

    $response = $this->getJson(route('maps.search', ['map' => $map, 'q' => 'tuskers']))
        ->assertSuccessful()
        ->json('occupiers');

    expect($response)->toHaveCount(1)
        ->and($response[0])->toEqual(['solarsystem_id' => 31000305, 'occupier_alias' => 'Tuskers Home']);
});
