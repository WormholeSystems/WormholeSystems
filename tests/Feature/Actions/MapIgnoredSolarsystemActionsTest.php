<?php

declare(strict_types=1);

use App\Actions\MapIgnoredSolarsystems\CreateMapIgnoredSolarsystemAction;
use App\Actions\MapIgnoredSolarsystems\DeleteMapIgnoredSolarsystemAction;
use App\Models\Map;
use App\Models\MapIgnoredSolarsystem;

it('ignores a solarsystem on a map', function () {
    $map = Map::factory()->create();
    $solarsystemId = makeSolarsystem(30005001);

    $ignored = app(CreateMapIgnoredSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'solarsystem_id' => $solarsystemId,
    ]);

    expect($ignored->map_id)->toBe($map->id)
        ->and($ignored->solarsystem_id)->toBe($solarsystemId)
        ->and(MapIgnoredSolarsystem::where('map_id', $map->id)->count())->toBe(1);
});

it('does not duplicate an ignored solarsystem', function () {
    $map = Map::factory()->create();
    $solarsystemId = makeSolarsystem(30005002);
    $data = ['map_id' => $map->id, 'solarsystem_id' => $solarsystemId];

    app(CreateMapIgnoredSolarsystemAction::class)->handle($data);
    app(CreateMapIgnoredSolarsystemAction::class)->handle($data);

    expect(MapIgnoredSolarsystem::where('map_id', $map->id)->count())->toBe(1);
});

it('stops ignoring a solarsystem', function () {
    $map = Map::factory()->create();
    $ignored = app(CreateMapIgnoredSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'solarsystem_id' => makeSolarsystem(30005003),
    ]);

    app(DeleteMapIgnoredSolarsystemAction::class)->handle($ignored);

    expect(MapIgnoredSolarsystem::find($ignored->id))->toBeNull();
});
