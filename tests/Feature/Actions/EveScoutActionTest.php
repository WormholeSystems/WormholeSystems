<?php

declare(strict_types=1);

use App\Actions\EveScout\AddEveScoutConnectionToMapAction;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\Solarsystem;
use Illuminate\Support\Facades\Http;

it('adds eve scout connections (and their systems) to the map', function () {
    $theraId = makeSolarsystem(31000005);
    $destinationId = makeSolarsystem(30002086);
    $map = Map::factory()->create();

    Http::fake(['*' => Http::response([
        [
            'in_system_id' => $theraId,
            'out_system_id' => $destinationId,
            'in_signature' => 'AAA-111',
            'out_signature' => 'BBB-222',
            'wh_type' => 'K162',
            'life' => 'stable',
            'mass' => 'stable',
            'remaining_hours' => 16,
            'completed' => true,
            'signature_type' => 'wormhole',
            'created_at' => null,
        ],
    ])]);

    app(AddEveScoutConnectionToMapAction::class)->handle($map, Solarsystem::find($theraId));

    expect($map->mapSolarsystems()->where('solarsystem_id', $theraId)->exists())->toBeTrue()
        ->and($map->mapSolarsystems()->where('solarsystem_id', $destinationId)->exists())->toBeTrue()
        ->and(MapConnection::where('map_id', $map->id)->count())->toBe(1);
});
