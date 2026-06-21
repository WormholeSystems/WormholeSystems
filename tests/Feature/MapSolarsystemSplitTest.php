<?php

declare(strict_types=1);

use App\Actions\MapSolarsystem\DeleteMapSolarsystemAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\MapSolarsystemDetails;
use App\Models\Signature;
use App\Models\User;

use function Pest\Laravel\actingAs;

it('hard-deletes the placement and cascades its connections and signatures, but keeps the details', function () {
    $map = Map::factory()->create();
    $originId = makeSolarsystem(30009601);
    $targetId = makeSolarsystem(30009602);

    $origin = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $originId]);
    $target = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $targetId]);

    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $origin->id,
        'to_map_solarsystem_id' => $target->id,
    ]);

    $signature = Signature::create([
        'map_solarsystem_id' => $target->id,
        'map_connection_id' => $connection->id,
        'signature_id' => 'ABC-123',
    ]);

    $detailsId = $target->map_solarsystem_details_id;

    app(DeleteMapSolarsystemAction::class)->handle($target);

    expect(MapSolarsystem::find($target->id))->toBeNull()
        ->and(MapConnection::find($connection->id))->toBeNull()
        ->and(Signature::find($signature->id))->toBeNull()
        ->and(MapSolarsystemDetails::find($detailsId))->not->toBeNull();
});

it('reuses surviving details and creates a fresh placement when a removed system is re-added', function () {
    $map = Map::factory()->create();
    $solarsystemId = makeSolarsystem(30009603);

    $placement = app(StoreMapSolarsystemAction::class)->handle($map, [
        'solarsystem_id' => $solarsystemId,
        'position_x' => 100,
        'position_y' => 100,
    ]);

    $detailsId = $placement->map_solarsystem_details_id;
    MapSolarsystemDetails::query()->whereKey($detailsId)->update(['occupier_alias' => 'Some Corp']);

    app(DeleteMapSolarsystemAction::class)->handle($placement);
    expect(MapSolarsystem::find($placement->id))->toBeNull();

    $readded = app(StoreMapSolarsystemAction::class)->handle($map, [
        'solarsystem_id' => $solarsystemId,
        'position_x' => 200,
        'position_y' => 200,
    ]);

    expect($readded->id)->not->toBe($placement->id)
        ->and($readded->map_solarsystem_details_id)->toBe($detailsId)
        ->and(MapSolarsystemDetails::find($detailsId)->occupier_alias)->toBe('Some Corp');
});

it('only ever holds placements that are on the map so no off-map connection can surface', function () {
    $map = Map::factory()->create();
    $aId = makeSolarsystem(30009604);
    $bId = makeSolarsystem(30009605);

    $a = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $aId]);
    $b = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $bId]);
    MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $a->id,
        'to_map_solarsystem_id' => $b->id,
    ]);

    app(DeleteMapSolarsystemAction::class)->handle($b);

    // The surviving system has no dangling connection to the removed one.
    expect(MapConnection::query()->where('map_id', $map->id)->count())->toBe(0)
        ->and(MapSolarsystem::query()->where('map_id', $map->id)->whereNull('position_x')->count())->toBe(0);
});

it('selects by solarsystem id and clears the selection instead of 404ing when the system is removed', function () {
    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();
    $user->update(['preferred_character_id' => $user->characters->first()->id]);
    $solarsystemId = makeSolarsystem(30009606);
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $solarsystemId]);

    actingAs($user);

    // The system can be selected by its solarsystem id while it is on the map.
    $this->get(route('maps.show', ['map' => $map, 'solarsystem_id' => $solarsystemId]))
        ->assertSuccessful();

    app(DeleteMapSolarsystemAction::class)->handle($placement);

    // After removal the same URL no longer 404s; the selection simply clears.
    $this->get(route('maps.show', ['map' => $map, 'solarsystem_id' => $solarsystemId]))
        ->assertSuccessful();
});
