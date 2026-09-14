<?php

declare(strict_types=1);

use App\Actions\EveScout\AddEveScoutConnectionToMapAction;
use App\Enums\ShipSize;
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

it('leaves the ship size unknown when the eve scout wormhole type is not one we know', function () {
    $theraId = makeSolarsystem(31000006);
    $destinationId = makeSolarsystem(30002087);
    $map = Map::factory()->create();

    Http::fake(['*' => Http::response([
        eveScoutConnection($theraId, $destinationId, 'Q003'),
    ])]);

    app(AddEveScoutConnectionToMapAction::class)->handle($map, Solarsystem::find($theraId));

    expect(MapConnection::where('map_id', $map->id)->value('ship_size'))->toBeNull();
});

it('derives the ship size from a known eve scout wormhole type', function () {
    $theraId = makeSolarsystem(31000007);
    $destinationId = makeSolarsystem(30002088);
    $map = Map::factory()->create();
    makeWormhole('E545', 5_000_000);

    Http::fake(['*' => Http::response([
        eveScoutConnection($theraId, $destinationId, 'E545'),
    ])]);

    app(AddEveScoutConnectionToMapAction::class)->handle($map, Solarsystem::find($theraId));

    expect(MapConnection::where('map_id', $map->id)->value('ship_size'))->toBe(ShipSize::Frigate);
});

/**
 * @return array<string, mixed>
 */
function eveScoutConnection(int $in_system_id, int $out_system_id, string $wh_type): array
{
    return [
        'in_system_id' => $in_system_id,
        'out_system_id' => $out_system_id,
        'in_signature' => 'AAA-111',
        'out_signature' => 'BBB-222',
        'wh_type' => $wh_type,
        'life' => 'stable',
        'mass' => 'stable',
        'remaining_hours' => 16,
        'completed' => true,
        'signature_type' => 'wormhole',
        'created_at' => null,
    ];
}
