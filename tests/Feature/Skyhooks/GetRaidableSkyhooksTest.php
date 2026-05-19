<?php

declare(strict_types=1);

use App\Jobs\Skyhooks\GetRaidableSkyhooks;
use App\Models\RaidableSkyhook;
use Illuminate\Support\Facades\DB;
use NicolasKion\Esi\DTO\EsiError;
use NicolasKion\Esi\DTO\EsiResult;
use NicolasKion\Esi\DTO\RaidableSkyhook as RaidableSkyhookDto;
use NicolasKion\Esi\DTO\SkyhookTheftVulnerability;
use NicolasKion\Esi\Esi;

function createSkyhookSolarsystem(int $id): void
{
    DB::table('regions')->insertOrIgnore([
        'id' => 10000099,
        'name' => 'Skyhook Region',
        'type' => 'normal',
    ]);

    DB::table('constellations')->insertOrIgnore([
        'id' => 20000099,
        'name' => 'Skyhook Constellation',
        'region_id' => 10000099,
        'type' => 'normal',
    ]);

    DB::table('solarsystems')->insertOrIgnore([
        'id' => $id,
        'name' => sprintf('Skyhook System %d', $id),
        'constellation_id' => 20000099,
        'region_id' => 10000099,
        'security' => 0.0,
        'pos_x' => 0,
        'pos_y' => 0,
        'pos_z' => 0,
        'type' => 'normal',
    ]);
}

function makeRaidableSkyhookDto(int $planetId, int $solarsystemId, string $start, string $end): RaidableSkyhookDto
{
    return new RaidableSkyhookDto(
        planet_id: $planetId,
        solar_system_id: $solarsystemId,
        theft_vulnerability: new SkyhookTheftVulnerability(start: $start, end: $end),
    );
}

it('upserts raidable skyhooks returned by ESI', function () {
    createSkyhookSolarsystem(30000201);
    createSkyhookSolarsystem(30000202);

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getRaidableSkyhooks')
        ->once()
        ->andReturn(new EsiResult(data: [
            makeRaidableSkyhookDto(40000001, 30000201, '2026-05-19T12:00:00Z', '2026-05-19T14:00:00Z'),
            makeRaidableSkyhookDto(40000002, 30000202, '2026-05-19T13:00:00Z', '2026-05-19T15:00:00Z'),
        ]));

    GetRaidableSkyhooks::dispatchSync();

    expect(RaidableSkyhook::query()->count())->toBe(2);

    $first = RaidableSkyhook::query()->find(40000001);
    expect($first)->not->toBeNull()
        ->solarsystem_id->toBe(30000201)
        ->theft_vulnerability_start->toDateTimeString()->toBe('2026-05-19 12:00:00')
        ->theft_vulnerability_end->toDateTimeString()->toBe('2026-05-19 14:00:00');
});

it('updates existing skyhooks in place', function () {
    createSkyhookSolarsystem(30000203);

    RaidableSkyhook::query()->create([
        'planet_id' => 40000003,
        'solarsystem_id' => 30000203,
        'theft_vulnerability_start' => '2026-05-19 08:00:00',
        'theft_vulnerability_end' => '2026-05-19 10:00:00',
    ]);

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getRaidableSkyhooks')
        ->once()
        ->andReturn(new EsiResult(data: [
            makeRaidableSkyhookDto(40000003, 30000203, '2026-05-19T20:00:00Z', '2026-05-19T22:00:00Z'),
        ]));

    GetRaidableSkyhooks::dispatchSync();

    expect(RaidableSkyhook::query()->count())->toBe(1);
    expect(RaidableSkyhook::query()->find(40000003))
        ->theft_vulnerability_start->toDateTimeString()->toBe('2026-05-19 20:00:00');
});

it('deletes skyhooks no longer present in ESI response', function () {
    createSkyhookSolarsystem(30000204);
    createSkyhookSolarsystem(30000205);

    RaidableSkyhook::query()->create([
        'planet_id' => 40000004,
        'solarsystem_id' => 30000204,
        'theft_vulnerability_start' => '2026-05-19 08:00:00',
        'theft_vulnerability_end' => '2026-05-19 10:00:00',
    ]);
    RaidableSkyhook::query()->create([
        'planet_id' => 40000005,
        'solarsystem_id' => 30000205,
        'theft_vulnerability_start' => '2026-05-19 09:00:00',
        'theft_vulnerability_end' => '2026-05-19 11:00:00',
    ]);

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getRaidableSkyhooks')
        ->once()
        ->andReturn(new EsiResult(data: [
            makeRaidableSkyhookDto(40000004, 30000204, '2026-05-19T12:00:00Z', '2026-05-19T14:00:00Z'),
        ]));

    GetRaidableSkyhooks::dispatchSync();

    expect(RaidableSkyhook::query()->pluck('planet_id')->all())->toBe([40000004]);
});

it('does nothing when ESI request fails', function () {
    createSkyhookSolarsystem(30000206);

    RaidableSkyhook::query()->create([
        'planet_id' => 40000006,
        'solarsystem_id' => 30000206,
        'theft_vulnerability_start' => '2026-05-19 08:00:00',
        'theft_vulnerability_end' => '2026-05-19 10:00:00',
    ]);

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getRaidableSkyhooks')
        ->once()
        ->andReturn(new EsiResult(error: new EsiError(code: 500, body: 'Server Error')));

    GetRaidableSkyhooks::dispatchSync();

    expect(RaidableSkyhook::query()->count())->toBe(1);
});
