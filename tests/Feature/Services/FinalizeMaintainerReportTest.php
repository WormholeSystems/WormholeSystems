<?php

declare(strict_types=1);

use App\Actions\Statistics\FinalizeMaintainerReportAction;
use App\Enums\SignatureActivityAction;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapMaintainerReport;
use App\Models\SignatureActivity;
use App\Services\Statistics\MaintainerPeriod;
use App\Services\Statistics\MaintainerReportReader;
use Carbon\CarbonImmutable;

use function Pest\Laravel\travelTo;

beforeEach(function () {
    $this->action = app(FinalizeMaintainerReportAction::class);
    $this->reader = app(MaintainerReportReader::class);
    $this->period = MaintainerPeriod::fromString('2026-06');

    travelTo(CarbonImmutable::parse('2026-07-15 00:00:00', 'UTC'));
});

it('throws when finalizing a period that has not ended yet', function () {
    $map = Map::factory()->create();

    $this->action->handle($map, MaintainerPeriod::current());
})->throws(InvalidArgumentException::class);

it('succeeds finalizing a period that ended exactly now', function () {
    $map = Map::factory()->create()->fresh();
    travelTo($this->period->endsAt());

    $report = $this->action->handle($map, $this->period);

    expect($report)->toBeInstanceOf(MapMaintainerReport::class);
});

it('stores the payload and freezes it against later weight changes, for both details and aggregated', function () {
    $map = Map::factory()->create([
        'maintainer_points_created' => 1,
        'maintainer_minimum_points' => 0,
    ])->fresh();
    $character = Character::factory()->create();

    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => 1,
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:1',
        'action' => SignatureActivityAction::Created,
        'activity_date' => '2026-06-15',
        'created_at' => CarbonImmutable::parse('2026-06-15', 'UTC'),
    ]);

    $this->action->handle($map, $this->period);

    $map->update(['maintainer_points_created' => 50]);

    $details = $this->reader->details($map->fresh(), $this->period);
    $aggregated = $this->reader->aggregated($map->fresh(), $this->period);

    expect($details->sole()->points)->toBe(1)
        ->and($aggregated->sole()->points)->toBe(1);
});

it('includes a sub-threshold character in both details and aggregated for a stored period', function () {
    $map = Map::factory()->create([
        'maintainer_points_created' => 1,
        'maintainer_minimum_points' => 5,
    ])->fresh();
    $character = Character::factory()->create();

    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => 1,
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:1',
        'action' => SignatureActivityAction::Created,
        'activity_date' => '2026-06-15',
        'created_at' => CarbonImmutable::parse('2026-06-15', 'UTC'),
    ]);

    $this->action->handle($map, $this->period);

    expect($this->reader->details($map, $this->period))->toHaveCount(1)
        ->and($this->reader->aggregated($map, $this->period))->toHaveCount(1);
});

it('keeps a sub-threshold scorer visible in a stored period regardless of later threshold changes', function () {
    $map = Map::factory()->create([
        'maintainer_points_created' => 1,
        'maintainer_minimum_points' => 5,
    ])->fresh();
    $character = Character::factory()->create();

    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => 1,
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:1',
        'action' => SignatureActivityAction::Created,
        'activity_date' => '2026-06-15',
        'created_at' => CarbonImmutable::parse('2026-06-15', 'UTC'),
    ]);

    $this->action->handle($map, $this->period);

    expect($this->reader->aggregated($map, $this->period))->toHaveCount(1);

    $map->update(['maintainer_minimum_points' => 100]);

    expect($this->reader->aggregated($map->fresh(), $this->period))->toHaveCount(1);
});

it('does not blow up on a contributor with a null character name', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $character = Character::factory()->create(['name' => null]);

    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => 1,
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:1',
        'action' => SignatureActivityAction::Created,
        'activity_date' => '2026-06-15',
        'created_at' => CarbonImmutable::parse('2026-06-15', 'UTC'),
    ]);

    $this->action->handle($map, $this->period);

    expect($this->reader->details($map, $this->period)->sole()->character_name)->toBe('Unknown character')
        ->and($this->reader->aggregated($map, $this->period)->sole()->display_name)->toBe('Unknown character');
});

it('is idempotent when finalized twice, and leaves an already-set discord_posted_at untouched', function () {
    $map = Map::factory()->create()->fresh();

    $first = $this->action->handle($map, $this->period);
    $first->forceFill(['discord_posted_at' => CarbonImmutable::parse('2026-07-01', 'UTC')])->save();

    $second = $this->action->handle($map, $this->period);

    expect(MapMaintainerReport::query()->where('map_id', $map->id)->where('period', '2026-06')->count())->toBe(1)
        ->and($second->id)->toBe($first->id)
        ->and($second->discord_posted_at)->not->toBeNull();
});

it('finalizes a finished period with no stored report on demand, without emptying it', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $character = Character::factory()->create();

    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => 1,
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:1',
        'action' => SignatureActivityAction::Created,
        'activity_date' => '2026-06-15',
        'created_at' => CarbonImmutable::parse('2026-06-15', 'UTC'),
    ]);

    expect(MapMaintainerReport::query()->where('map_id', $map->id)->where('period', '2026-06')->exists())->toBeFalse();

    $entries = $this->reader->aggregated($map, $this->period);

    expect($entries->sole()->points)->toBe(1);

    $report = MapMaintainerReport::query()->where('map_id', $map->id)->where('period', '2026-06')->firstOrFail();
    expect($report->discord_posted_at)->toBeNull();
});

it('freezes the period at the first on-demand read', function () {
    $map = Map::factory()->create([
        'maintainer_points_created' => 1,
        'maintainer_minimum_points' => 0,
    ])->fresh();
    $character = Character::factory()->create();

    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => 1,
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:1',
        'action' => SignatureActivityAction::Created,
        'activity_date' => '2026-06-15',
        'created_at' => CarbonImmutable::parse('2026-06-15', 'UTC'),
    ]);

    $first = $this->reader->details($map, $this->period);

    $map->update(['maintainer_points_created' => 50]);

    $second = $this->reader->details($map->fresh(), $this->period);

    expect($first->sole()->points)->toBe(1)
        ->and($second->sole()->points)->toBe(1);
});

it('returns the current period\'s live data without persisting a report', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $character = Character::factory()->create();

    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => 1,
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:1',
        'action' => SignatureActivityAction::Created,
        'activity_date' => CarbonImmutable::now('UTC')->toDateString(),
        'created_at' => CarbonImmutable::now('UTC'),
    ]);

    $entries = $this->reader->aggregated($map, MaintainerPeriod::current());

    expect($entries->sole()->points)->toBe(1)
        ->and(MapMaintainerReport::query()->count())->toBe(0);
});

it('returns exactly 13 selectable periods, newest first, starting at the current month', function () {
    $periods = MaintainerPeriod::selectable();

    expect($periods)->toHaveCount(13)
        ->and($periods[0]->toString())->toBe('2026-07')
        ->and($periods[1]->toString())->toBe('2026-06')
        ->and($periods[12]->toString())->toBe('2025-07');
});
