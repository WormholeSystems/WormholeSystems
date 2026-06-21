<?php

declare(strict_types=1);

use App\Actions\MapSelection\DeleteMapSelectionAction;
use App\Actions\MapSelection\UpdateMapSelectionAction;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\MapSolarsystemDetails;

it('repositions the selected systems', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30008001, 100, 100);

    app(UpdateMapSelectionAction::class)->handle([
        'map_solarsystems' => [
            ['id' => $system->id, 'position_x' => 500, 'position_y' => 600],
        ],
    ]);

    expect($system->fresh()->position_x)->toBe(500)
        ->and($system->fresh()->position_y)->toBe(600);
});

it('removes the selected systems and cascades their connections while keeping details', function () {
    $map = Map::factory()->create();
    $a = placeMapSolarsystem($map, 30008002);
    $b = placeMapSolarsystem($map, 30008003);
    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $a->id,
        'to_map_solarsystem_id' => $b->id,
    ]);

    app(DeleteMapSelectionAction::class)->handle([$a->id, $b->id]);

    expect(MapSolarsystem::whereIn('id', [$a->id, $b->id])->count())->toBe(0)
        ->and(MapConnection::find($connection->id))->toBeNull()
        ->and(MapSolarsystemDetails::where('map_id', $map->id)->count())->toBe(2);
});
