<?php

declare(strict_types=1);

use App\Actions\Tracking\StoreTrackingAction;
use App\Data\TrackingData;
use App\Models\Map;
use App\Models\MapConnection;

it('adds the target system and connects it when tracking a jump', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012001);
    $targetId = makeSolarsystem(30012002);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
    ]));

    expect(MapConnection::where('map_id', $map->id)->count())->toBe(1)
        ->and($map->mapSolarsystems()->where('solarsystem_id', $targetId)->whereNotNull('position_x')->exists())->toBeTrue();
});

it('does not duplicate a connection that already exists', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012003);
    $targetId = makeSolarsystem(30012004);

    $data = TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
    ]);

    app(StoreTrackingAction::class)->handle($data);
    app(StoreTrackingAction::class)->handle($data);

    expect(MapConnection::where('map_id', $map->id)->count())->toBe(1);
});
