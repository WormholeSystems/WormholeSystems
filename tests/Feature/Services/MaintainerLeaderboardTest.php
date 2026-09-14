<?php

declare(strict_types=1);

use App\Enums\SignatureActivityAction;
use App\Models\Character;
use App\Models\Map;
use App\Models\SignatureActivity;
use App\Models\User;
use App\Services\Statistics\MaintainerLeaderboard;
use App\Services\Statistics\MaintainerPeriod;
use Carbon\CarbonImmutable;

function recordMaintainerActivity(
    Map $map,
    Character $character,
    SignatureActivityAction $action,
    CarbonImmutable $at,
    int $signatureId,
    int $solarsystemId = 30000001,
): SignatureActivity {
    return SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => $signatureId,
        'solarsystem_id' => $solarsystemId,
        'dedupe_key' => 'pk:'.$signatureId,
        'action' => $action,
        'activity_date' => $at->toDateString(),
        'created_at' => $at,
    ]);
}

beforeEach(function () {
    $this->leaderboard = new MaintainerLeaderboard;
    $this->period = MaintainerPeriod::fromString('2026-06');
});

it('applies the map\'s configured weights, and a live computation reflects a later change', function () {
    $map = Map::factory()->create([
        'maintainer_points_created' => 5,
        'maintainer_points_updated' => 2,
        'maintainer_points_deleted' => 1,
        'maintainer_minimum_points' => 0,
    ]);
    $character = Character::factory()->create();
    $at = CarbonImmutable::parse('2026-06-15 12:00:00', 'UTC');

    recordMaintainerActivity($map, $character, SignatureActivityAction::Created, $at, 1);
    recordMaintainerActivity($map, $character, SignatureActivityAction::Updated, $at, 2);
    recordMaintainerActivity($map, $character, SignatureActivityAction::Deleted, $at, 3);

    $stat = $this->leaderboard->details($map, $this->period)->sole();
    expect($stat->points)->toBe(5 * 1 + 2 * 1 + 1 * 1);

    $map->update(['maintainer_points_created' => 10]);

    $stat = $this->leaderboard->details($map->fresh(), $this->period)->sole();
    expect($stat->points)->toBe(10 + 2 + 1);
});

it('sums alts under one user_id with a correct per-alt breakdown', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $user = User::factory()->create();
    $characterA = Character::factory()->for($user)->create();
    $characterB = Character::factory()->for($user)->create();
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($map, $characterA, SignatureActivityAction::Created, $at, 1);
    recordMaintainerActivity($map, $characterA, SignatureActivityAction::Created, $at, 2);
    recordMaintainerActivity($map, $characterB, SignatureActivityAction::Created, $at, 3);

    $entry = $this->leaderboard->aggregated($map, $this->period)->sole();

    expect($entry->points)->toBe(3)
        ->and($entry->user_id)->toBe($user->id)
        ->and($entry->characters)->toHaveCount(2);

    $statA = collect($entry->characters)->firstWhere('character_id', $characterA->id);
    $statB = collect($entry->characters)->firstWhere('character_id', $characterB->id);

    expect($statA->nb_added)->toBe(2)
        ->and($statB->nb_added)->toBe(1);
});

it('uses the preferred character as display_name when set', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $user = User::factory()->create();
    $preferred = Character::factory()->for($user)->create(['name' => 'Preferred Pilot']);
    $other = Character::factory()->for($user)->create(['name' => 'Other Pilot']);
    $user->forceFill(['preferred_character_id' => $preferred->id])->save();
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($map, $preferred, SignatureActivityAction::Created, $at, 1);
    recordMaintainerActivity($map, $other, SignatureActivityAction::Created, $at, 2);

    $entry = $this->leaderboard->aggregated($map, $this->period)->sole();

    expect($entry->display_name)->toBe('Preferred Pilot');
});

it('falls back to the top-scoring alt when no preferred character is set', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $user = User::factory()->create(['preferred_character_id' => null]);
    $weak = Character::factory()->for($user)->create(['name' => 'Weak Pilot']);
    $strong = Character::factory()->for($user)->create(['name' => 'Strong Pilot']);
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($map, $weak, SignatureActivityAction::Created, $at, 1);
    recordMaintainerActivity($map, $strong, SignatureActivityAction::Created, $at, 2);
    recordMaintainerActivity($map, $strong, SignatureActivityAction::Created, $at, 3);

    $entry = $this->leaderboard->aggregated($map, $this->period)->sole();

    expect($entry->display_name)->toBe('Strong Pilot');
});

it('falls back to the top-scoring alt when the preferred character scored nothing this period', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $user = User::factory()->create();
    $preferred = Character::factory()->for($user)->create(['name' => 'Idle Preferred']);
    $active = Character::factory()->for($user)->create(['name' => 'Active Alt']);
    $user->forceFill(['preferred_character_id' => $preferred->id])->save();
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($map, $active, SignatureActivityAction::Created, $at, 1);

    $entry = $this->leaderboard->aggregated($map, $this->period)->sole();

    expect($entry->display_name)->toBe('Active Alt');
});

it('ignores the map\'s minimum-points threshold -- everyone who scored appears', function () {
    $map = Map::factory()->create([
        'maintainer_points_created' => 1,
        'maintainer_minimum_points' => 100,
    ])->fresh();
    $character = Character::factory()->create();
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($map, $character, SignatureActivityAction::Created, $at, 1);

    $entries = $this->leaderboard->aggregated($map, $this->period);

    expect($entries)->toHaveCount(1)
        ->and($entries->first()->points)->toBe(1);
});

it('excludes a character with zero points from the aggregated leaderboard', function () {
    $map = Map::factory()->create([
        'maintainer_points_created' => 0,
        'maintainer_points_updated' => 0,
        'maintainer_points_deleted' => 0,
        'maintainer_minimum_points' => 0,
    ])->fresh();
    $character = Character::factory()->create();
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($map, $character, SignatureActivityAction::Created, $at, 1);

    $entries = $this->leaderboard->aggregated($map, $this->period);

    expect($entries)->toHaveCount(0);
});

it('gives a disassociated character its own entry', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $orphan = Character::factory()->create(['user_id' => null]);
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($map, $orphan, SignatureActivityAction::Created, $at, 1);

    $entry = $this->leaderboard->aggregated($map, $this->period)->sole();

    expect($entry->user_id)->toBeNull()
        ->and($entry->characters)->toHaveCount(1)
        ->and($entry->characters[0]->character_id)->toBe((int) $orphan->id);
});

it('buckets period boundaries on the UTC calendar month', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $character = Character::factory()->create();

    recordMaintainerActivity($map, $character, SignatureActivityAction::Created, CarbonImmutable::parse('2026-06-30 23:59:59', 'UTC'), 1);
    recordMaintainerActivity($map, $character, SignatureActivityAction::Created, CarbonImmutable::parse('2026-07-01 00:00:00', 'UTC'), 2);

    $june = $this->leaderboard->details($map, MaintainerPeriod::fromString('2026-06'))->sole();
    $july = $this->leaderboard->details($map, MaintainerPeriod::fromString('2026-07'))->sole();

    expect($june->nb_added)->toBe(1)
        ->and($july->nb_added)->toBe(1);
});

it('assigns stable 1..n positions with no gaps across repeated calls on tied points', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $characterA = Character::factory()->create();
    $characterB = Character::factory()->create();
    $characterC = Character::factory()->create();
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($map, $characterA, SignatureActivityAction::Created, $at, 1);
    recordMaintainerActivity($map, $characterB, SignatureActivityAction::Created, $at, 2);
    recordMaintainerActivity($map, $characterC, SignatureActivityAction::Created, $at, 3);
    recordMaintainerActivity($map, $characterC, SignatureActivityAction::Created, $at, 4);

    $first = $this->leaderboard->aggregated($map, $this->period);
    $second = $this->leaderboard->aggregated($map, $this->period);

    expect($first->pluck('position')->all())->toBe([1, 2, 3])
        ->and($first->pluck('position')->all())->toBe($second->pluck('position')->all())
        ->and($first->firstWhere('points', 2)->characters[0]->character_id)->toBe((int) $characterC->id);
});

it('does not blow up when a contributor\'s character name is null', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $character = Character::factory()->create(['name' => null]);
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($map, $character, SignatureActivityAction::Created, $at, 1);

    $detail = $this->leaderboard->details($map, $this->period)->sole();
    $entry = $this->leaderboard->aggregated($map, $this->period)->sole();

    expect($detail->character_name)->toBe('Unknown character')
        ->and($entry->display_name)->toBe('Unknown character');
});

it('scopes activity to the requested map', function () {
    $mapA = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $mapB = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $character = Character::factory()->create();
    $at = CarbonImmutable::parse('2026-06-10', 'UTC');

    recordMaintainerActivity($mapA, $character, SignatureActivityAction::Created, $at, 1);
    recordMaintainerActivity($mapA, $character, SignatureActivityAction::Created, $at, 2);
    recordMaintainerActivity($mapB, $character, SignatureActivityAction::Created, $at, 3);

    $statA = $this->leaderboard->details($mapA, $this->period)->sole();
    $statB = $this->leaderboard->details($mapB, $this->period)->sole();

    expect($statA->nb_added)->toBe(2)
        ->and($statB->nb_added)->toBe(1);
});

it('rejects a malformed period', function (string $value) {
    MaintainerPeriod::fromString($value);
})->throws(InvalidArgumentException::class)->with([
    '2026-13',
    '2026-00',
    'garbage',
    '2026-6',
    '26-06',
]);
