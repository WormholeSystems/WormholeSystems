<?php

declare(strict_types=1);

use App\Enums\LifetimeStatus;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\WormholeSystem;

use function Pest\Laravel\artisan;

function agingConnection(LifetimeStatus $lifetime, int $hoursOld, ?int $markedHoursAgo = null): MapConnection
{
    $map = Map::factory()->create();
    $wormhole = placeMapSolarsystem($map, 31000001);
    WormholeSystem::query()->create(['id' => 31000001, 'class' => '3']);
    $kspace = placeMapSolarsystem($map, 30004002);

    return MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $wormhole->id,
        'to_map_solarsystem_id' => $kspace->id,
        'lifetime' => $lifetime,
        'lifetime_updated_at' => $markedHoursAgo === null ? null : now()->subHours($markedHoursAgo),
        'connected_at' => now()->subHours($hoursOld),
        'created_at' => now()->subHours($hoursOld),
    ]);
}

it('ages a connection from healthy through end of life to critical', function (int $hoursOld, LifetimeStatus $expected) {
    $connection = agingConnection(LifetimeStatus::Healthy, $hoursOld);

    artisan('app:check-connection-age')->assertSuccessful();

    expect($connection->refresh()->lifetime)->toBe($expected);
})->with([
    'young' => [2, LifetimeStatus::Healthy],
    'end of life' => [21, LifetimeStatus::EndOfLife],
    'critical' => [23, LifetimeStatus::Critical],
]);

it('never ages a critical connection into expired', function () {
    $connection = agingConnection(LifetimeStatus::Critical, 30, markedHoursAgo: 5);

    artisan('app:check-connection-age')->assertSuccessful();

    expect($connection->refresh()->lifetime)->toBe(LifetimeStatus::Critical);
});

it('leaves an expired connection alone however young it looks', function () {
    $connection = agingConnection(LifetimeStatus::Expired, 1, markedHoursAgo: 0);

    artisan('app:check-connection-age')->assertSuccessful();

    expect($connection->refresh()->lifetime)->toBe(LifetimeStatus::Expired);
});

it('ranks expired as the most severe lifetime status', function () {
    $severities = array_map(
        fn (LifetimeStatus $status): int => $status->severity(),
        LifetimeStatus::cases(),
    );

    expect(LifetimeStatus::Expired->severity())->toBe(max($severities))
        ->and(LifetimeStatus::Expired->severity())->toBeGreaterThan(LifetimeStatus::Critical->severity());
});
