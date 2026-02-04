<?php

declare(strict_types=1);

use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\Killmail;
use Illuminate\Support\Facades\DB;
use NicolasKion\Esi\DTO\EsiError;
use NicolasKion\Esi\DTO\EsiResult;
use NicolasKion\Esi\Esi;

function createTestSolarsystem(): int
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

function createKillmailWithVictim(int $killmailId, int $solarsystemId, ?int $corporationId, ?int $allianceId): Killmail
{
    return Killmail::query()->create([
        'id' => $killmailId,
        'hash' => fake()->sha1(),
        'solarsystem_id' => $solarsystemId,
        'time' => now(),
        'data' => [
            'killmail_id' => $killmailId,
            'killmail_time' => now()->toIso8601String(),
            'solar_system_id' => $solarsystemId,
            'victim' => [
                'damage_taken' => 1000,
                'ship_type_id' => 587,
                'corporation_id' => $corporationId,
                'alliance_id' => $allianceId,
            ],
            'attackers' => [],
        ],
        'zkb' => [],
    ]);
}

it('creates missing corporations and alliances from killmail victims', function () {
    $solarsystemId = createTestSolarsystem();

    $missingCorpId = 98765432;
    $missingAllianceId = 99876543;

    createKillmailWithVictim(100001, $solarsystemId, $missingCorpId, $missingAllianceId);

    $esiCorporation = new NicolasKion\Esi\DTO\Corporation(
        alliance_id: null,
        ceo_id: 90000001,
        creator_id: 90000002,
        date_founded: '2003-01-01T00:00:00Z',
        description: 'Backfilled Corp',
        faction_id: null,
        home_station_id: null,
        member_count: 5,
        name: 'Backfilled Corp',
        shares: 1000,
        tax_rate: 0.1,
        ticker: 'BFIL',
        url: '',
        war_eligible: true,
    );

    $esiAlliance = new NicolasKion\Esi\DTO\Alliance(
        creator_corporation_id: 98000001,
        creator_id: 90000003,
        date_founded: '2005-01-01T00:00:00Z',
        name: 'Backfilled Alliance',
        ticker: 'BFAL',
        executor_corporation_id: 98000002,
        faction_id: null,
    );

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getCorporation')
        ->with($missingCorpId)
        ->once()
        ->andReturn(new EsiResult(data: $esiCorporation));
    $esi->shouldReceive('getAlliance')
        ->with($missingAllianceId)
        ->once()
        ->andReturn(new EsiResult(data: $esiAlliance));

    $this->artisan('app:backfill-killmail-organisations')
        ->assertSuccessful();

    expect(Corporation::query()->find($missingCorpId))
        ->not->toBeNull()
        ->name->toBe('Backfilled Corp');

    expect(Alliance::query()->find($missingAllianceId))
        ->not->toBeNull()
        ->name->toBe('Backfilled Alliance');
});

it('reports zero missing when all organisations exist', function () {
    $solarsystemId = createTestSolarsystem();

    $corporation = Corporation::factory()->create(['id' => 98000088]);
    $alliance = Alliance::factory()->create(['id' => 99000088]);

    createKillmailWithVictim(100002, $solarsystemId, $corporation->id, $alliance->id);

    $esi = $this->mock(Esi::class);
    $esi->shouldNotReceive('getCorporation');
    $esi->shouldNotReceive('getAlliance');

    $this->artisan('app:backfill-killmail-organisations')
        ->assertSuccessful();
});

it('handles null and zero victim IDs gracefully', function () {
    $solarsystemId = createTestSolarsystem();

    createKillmailWithVictim(100003, $solarsystemId, null, null);
    createKillmailWithVictim(100004, $solarsystemId, 0, 0);

    $esi = $this->mock(Esi::class);
    $esi->shouldNotReceive('getCorporation');
    $esi->shouldNotReceive('getAlliance');

    $this->artisan('app:backfill-killmail-organisations')
        ->assertSuccessful();
});

it('handles ESI failures gracefully during backfill', function () {
    $solarsystemId = createTestSolarsystem();

    $missingCorpId = 98765433;

    createKillmailWithVictim(100005, $solarsystemId, $missingCorpId, null);

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getCorporation')
        ->with($missingCorpId)
        ->once()
        ->andReturn(new EsiResult(error: new EsiError(code: 404, body: 'Not found')));

    $this->artisan('app:backfill-killmail-organisations')
        ->assertSuccessful();

    expect(Corporation::query()->find($missingCorpId))->toBeNull();
});
