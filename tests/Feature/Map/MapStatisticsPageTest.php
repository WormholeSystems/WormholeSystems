<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Enums\SignatureActivityAction;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\SignatureActivity;
use App\Models\User;
use Carbon\CarbonImmutable;
use Inertia\Testing\AssertableInertia as Assert;

use function Pest\Laravel\actingAs;
use function Pest\Laravel\travelTo;

function statisticsPageUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

function recordStatisticsPageActivity(Map $map, Character $character): void
{
    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => random_int(1, 1_000_000),
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:'.random_int(1, 1_000_000),
        'action' => SignatureActivityAction::Created,
        'activity_date' => CarbonImmutable::now('UTC')->toDateString(),
        'created_at' => CarbonImmutable::now('UTC'),
    ]);
}

beforeEach(function () {
    travelTo(CarbonImmutable::parse('2026-07-15 00:00:00', 'UTC'));
});

it('shows a member the leaderboard page with the expected props', function () {
    $map = Map::factory()->create(['maintainer_minimum_points' => 0])->fresh();
    User::factory()->ownsMap($map)->create();
    $user = statisticsPageUser($map, Permission::Member);
    $character = $user->characters()->sole();
    recordStatisticsPageActivity($map, $character);

    actingAs($user)
        ->get(route('maps.leaderboard.show', $map))
        ->assertSuccessful()
        ->assertInertia(fn (Assert $page) => $page
            ->component('maps/ShowLeaderboard')
            ->where('period', '2026-07')
            ->where('permission', 'member')
            ->has('available_periods', 13)
            ->has('entries', 1)
            ->where('entries.0.display_name', $character->name)
            ->where('entries.0.characters.0.character_id', (int) $character->id)
            ->where('entries.0.characters.0.nb_added', 1)
            ->etc());
});

it('allows a viewer to see the leaderboard page', function () {
    $map = Map::factory()->create();
    User::factory()->ownsMap($map)->create();

    actingAs(statisticsPageUser($map, Permission::Viewer))
        ->get(route('maps.leaderboard.show', $map))
        ->assertSuccessful();
});

it('forbids a user with no access to the map from seeing the leaderboard page', function () {
    $map = Map::factory()->create();
    User::factory()->ownsMap($map)->create();
    $user = User::factory()->create();

    actingAs($user)
        ->get(route('maps.leaderboard.show', $map))
        ->assertForbidden();
});

it('offers exactly 13 selectable periods, newest first', function () {
    $map = Map::factory()->create();
    User::factory()->ownsMap($map)->create();

    actingAs(statisticsPageUser($map, Permission::Member))
        ->get(route('maps.leaderboard.show', $map))
        ->assertInertia(fn (Assert $page) => $page
            ->has('available_periods', 13)
            ->where('available_periods.0.value', '2026-07')
            ->where('available_periods.0.is_current', true)
            ->etc());
});

it('404s a period outside the selectable window', function () {
    $map = Map::factory()->create();

    actingAs(statisticsPageUser($map, Permission::Member))
        ->get(route('maps.leaderboard.show', $map).'?period=2019-01')
        ->assertNotFound();
});
