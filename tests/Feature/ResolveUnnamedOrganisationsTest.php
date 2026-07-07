<?php

declare(strict_types=1);

use App\Models\Alliance;
use App\Models\Corporation;
use NicolasKion\Esi\DTO\EsiError;
use NicolasKion\Esi\DTO\EsiResult;
use NicolasKion\Esi\Esi;

function makeEsiCorporation(string $name): NicolasKion\Esi\DTO\Corporation
{
    return new NicolasKion\Esi\DTO\Corporation(
        alliance_id: null,
        ceo_id: 90000001,
        creator_id: 90000002,
        date_founded: '2003-01-01T00:00:00Z',
        description: 'Test corporation',
        faction_id: null,
        home_station_id: null,
        member_count: 10,
        name: $name,
        shares: 1000,
        tax_rate: 0.1,
        ticker: 'TCORP',
        url: 'https://example.com',
        war_eligible: true,
    );
}

it('resolves names for unnamed corporations', function () {
    $corporation = Corporation::factory()->create(['id' => 98000201, 'name' => null]);

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getCorporation')
        ->with($corporation->id)
        ->once()
        ->andReturn(new EsiResult(data: makeEsiCorporation('Resolved Corp')));

    $this->artisan('app:resolve-unnamed-organisations')->assertSuccessful();

    expect($corporation->refresh())
        ->name->toBe('Resolved Corp')
        ->unresolvable_at->toBeNull();
});

it('marks corporations that ESI answers 404 for as unresolvable and stops retrying them', function () {
    $corporation = Corporation::factory()->create(['id' => 98000202, 'name' => null]);

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getCorporation')
        ->with($corporation->id)
        ->once()
        ->andReturn(new EsiResult(error: new EsiError(404, 'Corporation not found')));

    $this->artisan('app:resolve-unnamed-organisations')->assertSuccessful();

    expect($corporation->refresh()->unresolvable_at)->not->toBeNull();

    $esi->shouldNotReceive('getCorporation');

    $this->artisan('app:resolve-unnamed-organisations')->assertSuccessful();
});

it('keeps retrying corporations after transient failures', function () {
    $corporation = Corporation::factory()->create(['id' => 98000203, 'name' => null]);

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getCorporation')
        ->with($corporation->id)
        ->twice()
        ->andReturn(new EsiResult(error: new EsiError(502, 'Bad gateway')));

    $this->artisan('app:resolve-unnamed-organisations')->assertSuccessful();

    expect($corporation->refresh()->unresolvable_at)->toBeNull();

    $this->artisan('app:resolve-unnamed-organisations')->assertSuccessful();
});

it('resolves unnamed alliances and marks 404s as unresolvable', function () {
    $resolvable = Alliance::factory()->create(['id' => 99000201, 'name' => null]);
    $deleted = Alliance::factory()->create(['id' => 99000202, 'name' => null]);

    $esiAlliance = new NicolasKion\Esi\DTO\Alliance(
        creator_corporation_id: 98000001,
        creator_id: 90000001,
        date_founded: '2010-01-01T00:00:00Z',
        executor_corporation_id: null,
        faction_id: null,
        name: 'Resolved Alliance',
        ticker: 'RSLVD',
    );

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getAlliance')->with($resolvable->id)->once()->andReturn(new EsiResult(data: $esiAlliance));
    $esi->shouldReceive('getAlliance')->with($deleted->id)->once()->andReturn(new EsiResult(error: new EsiError(404, 'Alliance not found')));

    $this->artisan('app:resolve-unnamed-organisations')->assertSuccessful();

    expect($resolvable->refresh())
        ->name->toBe('Resolved Alliance')
        ->unresolvable_at->toBeNull();

    expect($deleted->refresh()->unresolvable_at)->not->toBeNull();
});

it('skips named organisations entirely', function () {
    Corporation::factory()->create(['id' => 98000204, 'name' => 'Already Named']);
    Alliance::factory()->create(['id' => 99000204, 'name' => 'Already Named Alliance']);

    $esi = $this->mock(Esi::class);
    $esi->shouldNotReceive('getCorporation');
    $esi->shouldNotReceive('getAlliance');

    $this->artisan('app:resolve-unnamed-organisations')->assertSuccessful();
});
