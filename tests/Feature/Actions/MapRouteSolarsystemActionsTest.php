<?php

declare(strict_types=1);

use App\Actions\MapRouteSolarsystems\CreateMapRouteSolarsystemAction;
use App\Actions\MapRouteSolarsystems\DeleteMapRouteSolarsystemAction;
use App\Actions\MapRouteSolarsystems\UpdateMapRouteSolarsystemAction;
use App\Models\Map;
use App\Models\MapRouteSolarsystem;

it('adds a route waypoint to a map', function () {
    $map = Map::factory()->create();

    $route = app(CreateMapRouteSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'solarsystem_id' => makeSolarsystem(30006001),
        'is_pinned' => true,
    ]);

    expect($route->map_id)->toBe($map->id)
        ->and($route->is_pinned)->toBeTrue();
});

it('updates a route waypoint', function () {
    $map = Map::factory()->create();
    $route = app(CreateMapRouteSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'solarsystem_id' => makeSolarsystem(30006002),
        'is_pinned' => true,
    ]);

    app(UpdateMapRouteSolarsystemAction::class)->handle($route, ['is_pinned' => false]);

    expect($route->fresh()->is_pinned)->toBeFalse();
});

it('removes a route waypoint', function () {
    $map = Map::factory()->create();
    $route = app(CreateMapRouteSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'solarsystem_id' => makeSolarsystem(30006003),
        'is_pinned' => true,
    ]);

    app(DeleteMapRouteSolarsystemAction::class)->handle($route);

    expect(MapRouteSolarsystem::find($route->id))->toBeNull();
});
