<?php

declare(strict_types=1);

use App\Enums\LifetimeStatus;
use App\Enums\SolarsystemClass;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\WormholeSystem;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

function ageCheckConnection(Map $map, MapSolarsystem $from, MapSolarsystem $to, int $hoursAlive): MapConnection
{
    return MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        'lifetime' => LifetimeStatus::Healthy,
        'connected_at' => now()->subHours($hoursAlive),
    ]);
}

function ageCheckWormholeSystem(Map $map, int $solarsystemId, SolarsystemClass $class): MapSolarsystem
{
    $mapSolarsystem = placeMapSolarsystem($map, $solarsystemId);
    WormholeSystem::query()->forceCreate(['id' => $solarsystemId, 'class' => $class]);

    return $mapSolarsystem;
}

it('marks an aging wormhole connection as end of life', function () {
    $map = Map::factory()->create();
    $connection = ageCheckConnection(
        $map,
        ageCheckWormholeSystem($map, 31000001, SolarsystemClass::C3),
        ageCheckWormholeSystem($map, 31000002, SolarsystemClass::C5),
        hoursAlive: 21,
    );

    $this->artisan('app:check-connection-age')->assertSuccessful();

    expect($connection->refresh()->lifetime)->toBe(LifetimeStatus::EndOfLife);
});

it('gives a c6 to k-space connection its longer lifetime', function () {
    $map = Map::factory()->create();
    $connection = ageCheckConnection(
        $map,
        ageCheckWormholeSystem($map, 31000003, SolarsystemClass::C6),
        placeMapSolarsystem($map, 30000001),
        hoursAlive: 21,
    );

    $this->artisan('app:check-connection-age')->assertSuccessful();

    expect($connection->refresh()->lifetime)->toBe(LifetimeStatus::Healthy);
});

it('skips a connection whose map solarsystem no longer exists', function () {
    $map = Map::factory()->create();
    $wormhole = ageCheckWormholeSystem($map, 31000004, SolarsystemClass::C6);
    $kSpace = placeMapSolarsystem($map, 30000002);
    $orphaned = ageCheckConnection($map, $wormhole, $kSpace, hoursAlive: 47);
    $intact = ageCheckConnection($map, $wormhole, ageCheckWormholeSystem($map, 31000005, SolarsystemClass::C3), hoursAlive: 21);

    Schema::withoutForeignKeyConstraints(fn () => DB::table('map_solarsystems')->where('id', $kSpace->id)->delete());

    $this->artisan('app:check-connection-age')->assertSuccessful();

    expect($orphaned->refresh()->lifetime)->toBe(LifetimeStatus::Healthy)
        ->and($intact->refresh()->lifetime)->toBe(LifetimeStatus::EndOfLife);
});
