<?php

declare(strict_types=1);

use App\Actions\MapSolarsystem\UpdateMapSolarsystemAction;
use App\Enums\MapSolarsystemStatus;
use App\Models\Map;
use App\Models\MapSolarsystemDetails;

it('routes placement fields to the placement and intel fields to the details', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30009001);

    app(UpdateMapSolarsystemAction::class)->handle($system, [
        'alias' => 'Staging',
        'occupier_alias' => 'Some Corp',
        'status' => MapSolarsystemStatus::Friendly,
    ]);

    $details = MapSolarsystemDetails::find($system->map_solarsystem_details_id);

    expect($system->fresh()->alias)->toBe('Staging')
        ->and($details->occupier_alias)->toBe('Some Corp')
        ->and($details->status)->toBe(MapSolarsystemStatus::Friendly);
});
