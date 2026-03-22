<?php

declare(strict_types=1);

use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Support\Facades\DB;

use function Pest\Laravel\actingAs;

function createSolarsystem(): int
{
    $regionId = 10000001;
    $constellationId = 20000001;
    $solarsystemId = 30000001;

    DB::table('regions')->insertOrIgnore([
        'id' => $regionId,
        'name' => 'Test Region',
        'type' => 'normal',
    ]);

    DB::table('constellations')->insertOrIgnore([
        'id' => $constellationId,
        'name' => 'Test Constellation',
        'region_id' => $regionId,
        'type' => 'normal',
    ]);

    DB::table('solarsystems')->insertOrIgnore([
        'id' => $solarsystemId,
        'name' => 'Test System',
        'constellation_id' => $constellationId,
        'region_id' => $regionId,
        'security' => 0.5,
        'pos_x' => 0,
        'pos_y' => 0,
        'pos_z' => 0,
        'type' => 'normal',
    ]);

    return $solarsystemId;
}

it('returns maps with signatures count on index', function () {

    $solarsystemId = createSolarsystem();

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    MapSolarsystem::factory()->for($map)->create([
        'solarsystem_id' => $solarsystemId,
    ]);

    actingAs($user);

    $this->getJson(route('api.maps.index'))
        ->assertSuccessful()
        ->assertJsonStructure([
            'data' => [
                '*' => [
                    'map_solarsystems' => [
                        '*' => [
                            'signatures_count',
                            'wormhole_signatures_count',
                            'map_connections_count',
                        ],
                    ],
                ],
            ],
        ]);
});

it('returns a map with signatures count on show', function () {

    $solarsystemId = createSolarsystem();

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    MapSolarsystem::factory()->for($map)->create([
        'solarsystem_id' => $solarsystemId,
    ]);

    actingAs($user);

    $this->getJson(route('api.maps.show', $map))
        ->assertSuccessful()
        ->assertJsonStructure([
            'data' => [
                'map_solarsystems' => [
                    '*' => [
                        'signatures_count',
                        'wormhole_signatures_count',
                        'map_connections_count',
                    ],
                ],
            ],
        ]);
});
