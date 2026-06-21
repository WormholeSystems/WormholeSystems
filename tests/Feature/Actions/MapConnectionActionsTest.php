<?php

declare(strict_types=1);

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapConnections\CreateMapConnectionWithIDsAction;
use App\Actions\MapConnections\DeleteMapConnectionAction;
use App\Actions\MapConnections\UpdateMapConnectionAction;
use App\Data\MapConnectionData;
use App\Enums\MassStatus;
use App\Models\Map;
use App\Models\MapConnection;

it('creates a connection between two placements', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 30004001);
    $to = placeMapSolarsystem($map, 30004002);

    $connection = app(CreateMapConnectionAction::class)->handle([
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
    ]);

    expect($connection->map_id)->toBe($map->id)
        ->and($connection->from_map_solarsystem_id)->toBe($from->id)
        ->and($connection->to_map_solarsystem_id)->toBe($to->id);
});

it('creates a connection from solarsystem ids', function () {
    $map = Map::factory()->create();
    placeMapSolarsystem($map, 30004003);
    placeMapSolarsystem($map, 30004004);

    app(CreateMapConnectionWithIDsAction::class)->handle($map, 30004003, 30004004);

    expect(MapConnection::where('map_id', $map->id)->count())->toBe(1);
});

it('updates a connection mass status', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 30004005);
    $to = placeMapSolarsystem($map, 30004006);
    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        'mass_status' => MassStatus::Fresh,
    ]);

    app(UpdateMapConnectionAction::class)->handle($connection, MapConnectionData::from(['mass_status' => MassStatus::Critical]));

    expect($connection->fresh()->mass_status)->toBe(MassStatus::Critical);
});

it('deletes a connection', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 30004007);
    $to = placeMapSolarsystem($map, 30004008);
    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
    ]);

    app(DeleteMapConnectionAction::class)->handle($connection);

    expect(MapConnection::find($connection->id))->toBeNull();
});
