<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Enums\SignatureActivityAction;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapMaintainerReport;
use App\Models\SignatureActivity;
use App\Models\User;
use Carbon\CarbonImmutable;

use function Pest\Laravel\actingAs;
use function Pest\Laravel\getJson;
use function Pest\Laravel\travelTo;

function statsUser(Map $map, Permission $permission = Permission::Member): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

function recordStatsActivity(Map $map, Character $character, ?string $activityDate = null, SignatureActivityAction $action = SignatureActivityAction::Created): void
{
    $activityDate ??= CarbonImmutable::now('UTC')->toDateString();

    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => random_int(1, 1_000_000),
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:'.random_int(1, 1_000_000),
        'action' => $action,
        'activity_date' => $activityDate,
        'created_at' => CarbonImmutable::parse($activityDate, 'UTC'),
    ]);
}

beforeEach(function () {
    travelTo(CarbonImmutable::parse('2026-07-15 00:00:00', 'UTC'));
});

it('returns the aggregated shape, including the data wrapper, for a member', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $user = statsUser($map);
    recordStatsActivity($map, $user->characters()->sole());

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', $map))
        ->assertSuccessful()
        ->assertJsonStructure([
            'data' => [
                '*' => ['points', 'position', 'user_id', 'characters' => [['id', 'name']]],
            ],
        ]);
});

it('returns the details shape, including the data wrapper, for a member', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $user = statsUser($map);
    recordStatsActivity($map, $user->characters()->sole());

    actingAs($user)
        ->getJson(route('api.maps.stats.details', $map))
        ->assertSuccessful()
        ->assertJsonStructure([
            'data' => [
                '*' => ['character_id', 'character_name', 'user_id', 'nb_added', 'nb_edited', 'nb_deleted', 'points'],
            ],
        ]);
});

it('allows a viewer to read the stats endpoints', function (string $route) {
    $map = Map::factory()->create()->fresh();
    $user = statsUser($map, Permission::Viewer);

    actingAs($user)
        ->getJson(route($route, $map))
        ->assertSuccessful();
})->with([
    'aggregated' => ['api.maps.stats.aggregated'],
    'details' => ['api.maps.stats.details'],
]);

it('rejects an unauthenticated request to a public map instead of returning data', function () {
    $map = Map::factory()->create(['is_public' => true])->fresh();

    getJson(route('api.maps.stats.aggregated', $map))
        ->assertUnauthorized();
});

it('forbids an authenticated user with no map access from reading a public map\'s stats', function () {
    // The stats endpoints gate on `viewLeaderboard` (any accessor row, viewer+), not
    // `view` -- `view` admits anonymous share-token/public-map visitors, but the stats
    // endpoints must not, since they expose character names and per-alt points.
    $map = Map::factory()->create(['is_public' => true])->fresh();
    $user = User::factory()->create();

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', $map))
        ->assertForbidden();
});

it('defaults to the current month when period is omitted', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $user = statsUser($map);
    recordStatsActivity($map, $user->characters()->sole(), '2026-07-15');

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', $map))
        ->assertSuccessful()
        ->assertJsonCount(1, 'data');
});

it('rejects an unparseable period with 422', function () {
    $map = Map::factory()->create()->fresh();
    $user = statsUser($map);

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => 'garbage']))
        ->assertStatus(422);
});

it('rejects a period older than the selectable window with 404', function () {
    $map = Map::factory()->create()->fresh();
    $user = statsUser($map);

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => '2019-01']))
        ->assertNotFound();
});

it('rejects a future period with 404', function () {
    $map = Map::factory()->create()->fresh();
    $user = statsUser($map);

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => '2030-01']))
        ->assertNotFound();
});

it('pins the selectable window boundary at exactly 13 months', function () {
    // Time is frozen at 2026-07-15, so selectable() spans 2025-07..2026-07 (13 entries).
    // This is the off-by-one the plan singled out -- selectable() must not be "12 back".
    $map = Map::factory()->create()->fresh();
    $user = statsUser($map);

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => '2025-07']))
        ->assertSuccessful();

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => '2025-06']))
        ->assertNotFound();
});

it('rejects an array period value with 422 instead of a server error', function () {
    $map = Map::factory()->create()->fresh();
    $user = statsUser($map);

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => ['2026-06']]))
        ->assertStatus(422);
});

it('reads a past period from the stored report rather than recomputing it', function () {
    $map = Map::factory()->create([
        'maintainer_points_created' => 1,
        'maintainer_minimum_points' => 0,
    ])->fresh();
    $user = statsUser($map);
    recordStatsActivity($map, $user->characters()->sole(), '2026-06-15');

    $first = actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => '2026-06']))
        ->assertSuccessful()
        ->json('data.0.points');

    $map->update(['maintainer_points_created' => 50]);

    $second = actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => '2026-06']))
        ->assertSuccessful()
        ->json('data.0.points');

    expect($second)->toBe($first);
});

it('includes a sub-threshold character in both aggregated and details for a stored past period', function () {
    $map = Map::factory()->create([
        'maintainer_points_created' => 1,
        'maintainer_minimum_points' => 5,
    ])->fresh();
    $user = statsUser($map);
    recordStatsActivity($map, $user->characters()->sole(), '2026-06-15');

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => '2026-06']))
        ->assertSuccessful()
        ->assertJsonCount(1, 'data');

    actingAs($user)
        ->getJson(route('api.maps.stats.details', ['map' => $map, 'period' => '2026-06']))
        ->assertSuccessful()
        ->assertJsonCount(1, 'data');
});

it('finalizes a past period with no stored report on demand and returns the real data', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    $user = statsUser($map);
    recordStatsActivity($map, $user->characters()->sole(), '2026-06-15');

    expect(MapMaintainerReport::query()->where('map_id', $map->id)->where('period', '2026-06')->exists())->toBeFalse();

    actingAs($user)
        ->getJson(route('api.maps.stats.aggregated', ['map' => $map, 'period' => '2026-06']))
        ->assertSuccessful()
        ->assertJsonCount(1, 'data');

    expect(MapMaintainerReport::query()->where('map_id', $map->id)->where('period', '2026-06')->exists())->toBeTrue();
});
