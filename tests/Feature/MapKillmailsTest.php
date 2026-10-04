<?php

declare(strict_types=1);

use App\Enums\KillmailFilter;
use App\Models\Killmail;
use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\MapUserSetting;
use App\Models\User;
use Illuminate\Support\Facades\DB;

use function Pest\Laravel\actingAs;

/**
 * @param  list<int>  $ids
 */
function makeMapKillmails(array $ids, int $solarsystemId): void
{
    Killmail::query()->insert(array_map(fn (int $id): array => [
        'id' => $id,
        'hash' => 'hash-'.$id,
        'solarsystem_id' => $solarsystemId,
        'time' => now(),
        'data' => json_encode([
            'victim' => [
                'character_id' => 90000001,
                'corporation_id' => null,
                'alliance_id' => null,
                'faction_id' => null,
                'ship_type_id' => 670,
                'items' => [],
            ],
            'attackers' => [],
        ]),
        'zkb' => json_encode(['totalValue' => 10_000_000, 'points' => 1]),
    ], $ids));
}

beforeEach(function () {
    $this->map = Map::factory()->create();
    $this->user = User::factory()->ownsMap($this->map)->create();
    $this->user->update(['preferred_character_id' => $this->user->characters->first()->id]);

    actingAs($this->user);

    $this->wormholeSystem = makeSolarsystem(31000001, -1.0, 'wh');
    $this->knownSpaceSystem = makeSolarsystem(30000142, 0.9, 'eve');
    $this->offMapSystem = makeSolarsystem(30000144, 0.9, 'eve');

    MapSolarsystem::factory()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->wormholeSystem]);
    MapSolarsystem::factory()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->knownSpaceSystem]);
});

/**
 * @param  array<string, int>  $query
 * @return list<int>
 */
function loadMapKillmailIds(Map $map, array $query = []): array
{
    $ids = [];

    test()->get(route('maps.show', ['map' => $map, ...$query]))
        ->assertSuccessful()
        ->assertInertia(function ($page) use (&$ids): void {
            $page->loadDeferredProps(function ($reload) use (&$ids): void {
                $ids = array_column($reload->toArray()['props']['map_killmails'], 'id');
            });
        });

    return $ids;
}

it('returns the fifty newest killmails across the systems on the map', function () {
    makeMapKillmails(range(1, 60), $this->knownSpaceSystem);
    makeMapKillmails(range(61, 64), $this->wormholeSystem);
    makeMapKillmails(range(100, 110), $this->offMapSystem);

    $ids = loadMapKillmailIds($this->map);

    expect($ids)->toBe(range(64, 15, -1));
});

it('only returns killmails from the filtered space type', function (KillmailFilter $filter, array $expected) {
    MapUserSetting::query()->updateOrCreate(
        ['user_id' => $this->user->id, 'map_id' => $this->map->id],
        ['killmail_filter' => $filter],
    );

    makeMapKillmails([1, 2, 3], $this->knownSpaceSystem);
    makeMapKillmails([4, 5], $this->wormholeSystem);

    expect(loadMapKillmailIds($this->map))->toBe($expected);
})->with([
    'j-space' => [KillmailFilter::JSpace, [5, 4]],
    'k-space' => [KillmailFilter::KSpace, [3, 2, 1]],
]);

it('returns no killmails for a map without systems', function () {
    DB::table('map_solarsystems')->where('map_id', $this->map->id)->delete();
    makeMapKillmails([1, 2], $this->knownSpaceSystem);

    expect(loadMapKillmailIds($this->map))->toBe([]);
});

it('runs one bounded lookup per system instead of scanning every killmail', function () {
    makeMapKillmails(range(1, 5), $this->knownSpaceSystem);

    $queries = [];
    DB::listen(function ($query) use (&$queries): void {
        if (str_contains($query->sql, 'from `killmails`')) {
            $queries[] = $query->sql;
        }
    });

    loadMapKillmailIds($this->map);

    $unionQuery = collect($queries)->first(fn (string $sql): bool => str_contains($sql, 'union all'));

    expect($unionQuery)->not->toBeNull()
        ->and(mb_substr_count($unionQuery, 'where `solarsystem_id` = ?'))->toBe(2);
});

it('accepts the selected system killmail filter', function () {
    $this->putJson(route('maps.user-settings.update', $this->map), [
        'killmail_filter' => 'selected_system',
    ])->assertRedirect();

    $settings = MapUserSetting::query()->where('user_id', $this->user->id)->where('map_id', $this->map->id)->sole();

    expect($settings->killmail_filter)->toBe(KillmailFilter::SelectedSystem);
});

it('only returns killmails from the selected system with the selected system filter', function () {
    MapUserSetting::query()->updateOrCreate(
        ['user_id' => $this->user->id, 'map_id' => $this->map->id],
        ['killmail_filter' => KillmailFilter::SelectedSystem],
    );

    makeMapKillmails([1, 2, 3], $this->knownSpaceSystem);
    makeMapKillmails([4, 5], $this->wormholeSystem);

    expect(loadMapKillmailIds($this->map, ['solarsystem_id' => $this->knownSpaceSystem]))->toBe([3, 2, 1])
        ->and(loadMapKillmailIds($this->map, ['solarsystem_id' => $this->wormholeSystem]))->toBe([5, 4]);
});

it('returns no killmails with the selected system filter when the selection is missing or off the map', function (?string $selection) {
    MapUserSetting::query()->updateOrCreate(
        ['user_id' => $this->user->id, 'map_id' => $this->map->id],
        ['killmail_filter' => KillmailFilter::SelectedSystem],
    );

    makeMapKillmails([1, 2], $this->knownSpaceSystem);
    makeMapKillmails([3], $this->offMapSystem);

    $query = $selection === null ? [] : ['solarsystem_id' => $this->{$selection}];

    expect(loadMapKillmailIds($this->map, $query))->toBe([]);
})->with([
    'nothing selected' => [null],
    'selected system off the map' => ['offMapSystem'],
]);
