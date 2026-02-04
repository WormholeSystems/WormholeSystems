<?php

declare(strict_types=1);

use App\Actions\EnsureOrganisationExistsAction;
use App\Models\Alliance;
use App\Models\Character;
use App\Models\Corporation;
use NicolasKion\Esi\DTO\EsiError;
use NicolasKion\Esi\DTO\EsiResult;
use NicolasKion\Esi\Esi;

it('creates a corporation when it does not exist', function () {
    $corporationId = 98000001;

    $esiCorporation = new NicolasKion\Esi\DTO\Corporation(
        alliance_id: null,
        ceo_id: 90000001,
        creator_id: 90000002,
        date_founded: '2003-01-01T00:00:00Z',
        description: 'Test corporation',
        faction_id: null,
        home_station_id: null,
        member_count: 10,
        name: 'Test Corp',
        shares: 1000,
        tax_rate: 0.1,
        ticker: 'TCORP',
        url: 'https://example.com',
        war_eligible: true,
    );

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getCorporation')
        ->with($corporationId)
        ->once()
        ->andReturn(new EsiResult(data: $esiCorporation));

    $action = app(EnsureOrganisationExistsAction::class);
    $action->ensureCorporationExists($corporationId);

    expect(Corporation::query()->find($corporationId))
        ->not->toBeNull()
        ->name->toBe('Test Corp')
        ->ticker->toBe('TCORP');

    expect(Character::query()->find(90000001))->not->toBeNull();
    expect(Character::query()->find(90000002))->not->toBeNull();
});

it('skips corporation creation when it already exists', function () {
    $corporation = Corporation::factory()->create(['id' => 98000099]);

    $esi = $this->mock(Esi::class);
    $esi->shouldNotReceive('getCorporation');

    $action = app(EnsureOrganisationExistsAction::class);
    $action->ensureCorporationExists($corporation->id);
});

it('creates an alliance when it does not exist', function () {
    $allianceId = 99000001;

    $esiAlliance = new NicolasKion\Esi\DTO\Alliance(
        creator_corporation_id: 98000001,
        creator_id: 90000001,
        date_founded: '2003-01-01T00:00:00Z',
        name: 'Test Alliance',
        ticker: 'TALLI',
        executor_corporation_id: 98000002,
        faction_id: null,
    );

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getAlliance')
        ->with($allianceId)
        ->once()
        ->andReturn(new EsiResult(data: $esiAlliance));

    $action = app(EnsureOrganisationExistsAction::class);
    $action->ensureAllianceExists($allianceId);

    expect(Alliance::query()->find($allianceId))
        ->not->toBeNull()
        ->name->toBe('Test Alliance')
        ->ticker->toBe('TALLI');

    expect(Character::query()->find(90000001))->not->toBeNull();
    expect(Corporation::query()->find(98000002))->not->toBeNull();
});

it('skips alliance creation when it already exists', function () {
    $alliance = Alliance::factory()->create(['id' => 99000099]);

    $esi = $this->mock(Esi::class);
    $esi->shouldNotReceive('getAlliance');

    $action = app(EnsureOrganisationExistsAction::class);
    $action->ensureAllianceExists($alliance->id);
});

it('handles null and zero IDs gracefully', function () {
    $esi = $this->mock(Esi::class);
    $esi->shouldNotReceive('getCorporation');
    $esi->shouldNotReceive('getAlliance');

    $action = app(EnsureOrganisationExistsAction::class);

    $action->ensureCorporationExists(null);
    $action->ensureCorporationExists(0);
    $action->ensureAllianceExists(null);
    $action->ensureAllianceExists(0);
});

it('handles ESI failures without throwing for corporation', function () {
    $corporationId = 98000001;

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getCorporation')
        ->with($corporationId)
        ->once()
        ->andReturn(new EsiResult(error: new EsiError(code: 404, body: 'Not found')));

    $action = app(EnsureOrganisationExistsAction::class);
    $action->ensureCorporationExists($corporationId);

    expect(Corporation::query()->find($corporationId))->toBeNull();
});

it('handles ESI failures without throwing for alliance', function () {
    $allianceId = 99000001;

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getAlliance')
        ->with($allianceId)
        ->once()
        ->andReturn(new EsiResult(error: new EsiError(code: 404, body: 'Not found')));

    $action = app(EnsureOrganisationExistsAction::class);
    $action->ensureAllianceExists($allianceId);

    expect(Alliance::query()->find($allianceId))->toBeNull();
});
