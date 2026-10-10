<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapRouteSolarsystem;
use App\Models\User;
use Illuminate\Support\Arr;
use Illuminate\Testing\TestResponse;

use function Pest\Laravel\actingAs;

function watchlistUser(Map $map, Permission $permission, bool $expired = false): User
{
    $access = MapAccess::factory(['permission' => $permission])->for($map);

    $user = User::factory()
        ->has(Character::factory()->has($expired ? $access->expired() : $access))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

/**
 * @return array{destinations: list<array<string, mixed>>, can_add_personal: bool, can_add_shared: bool}
 */
function watchlistNavigation(Map $map, array $query = []): array
{
    $navigation = [];

    test()->get(route('maps.show', ['map' => $map, ...$query]))
        ->assertSuccessful()
        ->assertInertia(function ($page) use (&$navigation): void {
            $navigation = $page->toArray()['props']['map_navigation'];
        });

    return $navigation;
}

function storeWatchlistEntry(Map $map, int $solarsystemId, array $extra = []): TestResponse
{
    return test()->post(route('map-route-solarsystems.store'), [
        'map_id' => $map->id,
        'solarsystem_id' => $solarsystemId,
        ...$extra,
    ]);
}

beforeEach(function () {
    $this->map = Map::factory()->create();
    User::factory()->ownsMap($this->map)->create();
    $this->solarsystemId = makeSolarsystem(30000142);
});

describe('personal watchlist', function () {
    it('lets any accessor create, pin, unpin and delete a personal entry', function (Permission $permission) {
        $user = watchlistUser($this->map, $permission);
        actingAs($user);

        storeWatchlistEntry($this->map, $this->solarsystemId)->assertRedirect();

        $entry = MapRouteSolarsystem::query()->where('map_id', $this->map->id)->sole();
        expect($entry->user_id)->toBe($user->id)
            ->and($entry->is_pinned)->toBeFalse();

        $this->put(route('map-route-solarsystems.update', $entry), ['is_pinned' => true])->assertRedirect();
        expect($entry->fresh()->is_pinned)->toBeTrue();

        $this->put(route('map-route-solarsystems.update', $entry), ['is_pinned' => false])->assertRedirect();
        expect($entry->fresh()->is_pinned)->toBeFalse();

        $this->delete(route('map-route-solarsystems.destroy', $entry))->assertRedirect();
        expect(MapRouteSolarsystem::query()->whereKey($entry->id)->exists())->toBeFalse();
    })->with([Permission::Viewer, Permission::Member, Permission::Manager]);

    it('ignores a user_id sent in the payload', function () {
        $user = watchlistUser($this->map, Permission::Viewer);
        $other = User::factory()->create();
        actingAs($user);

        storeWatchlistEntry($this->map, $this->solarsystemId, ['user_id' => $other->id])->assertRedirect();

        expect(MapRouteSolarsystem::query()->sole()->user_id)->toBe($user->id);
    });

    it('is idempotent for the same personal system', function () {
        actingAs(watchlistUser($this->map, Permission::Viewer));

        storeWatchlistEntry($this->map, $this->solarsystemId)->assertRedirect();
        storeWatchlistEntry($this->map, $this->solarsystemId)->assertRedirect();

        expect(MapRouteSolarsystem::query()->count())->toBe(1);
    });

    it('forbids editing another user personal entry', function (Permission $permission) {
        $entry = MapRouteSolarsystem::factory()->personal(watchlistUser($this->map, Permission::Viewer))
            ->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);
        actingAs(watchlistUser($this->map, $permission));

        $this->put(route('map-route-solarsystems.update', $entry), ['is_pinned' => true])->assertForbidden();
        $this->delete(route('map-route-solarsystems.destroy', $entry))->assertForbidden();

        expect($entry->fresh()->is_pinned)->toBeFalse();
    })->with([Permission::Viewer, Permission::Manager]);

    it('never shows another user personal entries', function () {
        $owner = watchlistUser($this->map, Permission::Viewer);
        MapRouteSolarsystem::factory()->personal($owner)->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);
        actingAs(watchlistUser($this->map, Permission::Manager));

        expect(watchlistNavigation($this->map)['destinations'])->toBe([]);
    });

    it('lets a user with expired access neither add nor edit their old entries', function () {
        $user = watchlistUser($this->map, Permission::Manager, expired: true);
        $entry = MapRouteSolarsystem::factory()->personal($user)
            ->create(['map_id' => $this->map->id, 'solarsystem_id' => makeSolarsystem(30002187)]);
        actingAs($user);

        storeWatchlistEntry($this->map, $this->solarsystemId)->assertForbidden();
        $this->put(route('map-route-solarsystems.update', $entry), ['is_pinned' => true])->assertForbidden();
        $this->delete(route('map-route-solarsystems.destroy', $entry))->assertForbidden();

        expect(MapRouteSolarsystem::query()->count())->toBe(1);
    });
});

describe('visitors without an accessor row', function () {
    it('redirects anonymous visitors to login', function () {
        storeWatchlistEntry($this->map, $this->solarsystemId)->assertRedirect(route('login'));

        expect(MapRouteSolarsystem::query()->count())->toBe(0);
    });

    it('forbids an authenticated visitor of a public map', function () {
        $this->map->update(['is_public' => true]);
        actingAs(User::factory()->create());

        storeWatchlistEntry($this->map, $this->solarsystemId)->assertForbidden();
        storeWatchlistEntry($this->map, $this->solarsystemId, ['is_shared' => true])->assertForbidden();

        expect(MapRouteSolarsystem::query()->count())->toBe(0);
    });

    it('gives public-map visitors the shared list only and no add rights', function () {
        $this->map->update(['is_public' => true]);
        MapRouteSolarsystem::factory()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);
        MapRouteSolarsystem::factory()->personal()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);

        $navigation = watchlistNavigation($this->map);

        expect($navigation['can_add_personal'])->toBeFalse()
            ->and($navigation['can_add_shared'])->toBeFalse()
            ->and($navigation['destinations'])->toHaveCount(1)
            ->and($navigation['destinations'][0]['is_personal'])->toBeFalse()
            ->and($navigation['destinations'][0]['can_edit'])->toBeFalse();

        actingAs(watchlistUser(Map::factory()->create(), Permission::Manager));
        expect(watchlistNavigation($this->map)['can_add_personal'])->toBeFalse();
    });

    it('gives share-token visitors no add rights', function () {
        $this->map->update(['share_token' => 'watchlist-token']);

        $navigation = watchlistNavigation($this->map, ['share_token' => 'watchlist-token']);

        expect($navigation['can_add_personal'])->toBeFalse()
            ->and($navigation['can_add_shared'])->toBeFalse();
    });
});

describe('shared watchlist', function () {
    it('forbids viewers and members from editing the shared list', function (Permission $permission) {
        $entry = MapRouteSolarsystem::factory()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);
        actingAs(watchlistUser($this->map, $permission));

        storeWatchlistEntry($this->map, makeSolarsystem(30002187), ['is_shared' => true])->assertForbidden();
        $this->put(route('map-route-solarsystems.update', $entry), ['is_pinned' => true])->assertForbidden();
        $this->delete(route('map-route-solarsystems.destroy', $entry))->assertForbidden();

        expect(MapRouteSolarsystem::query()->count())->toBe(1)
            ->and($entry->fresh()->is_pinned)->toBeFalse();
    })->with([Permission::Viewer, Permission::Member]);

    it('lets managers create, pin and delete shared entries', function () {
        actingAs(watchlistUser($this->map, Permission::Manager));

        storeWatchlistEntry($this->map, $this->solarsystemId, ['is_shared' => true])->assertRedirect();
        storeWatchlistEntry($this->map, $this->solarsystemId, ['is_shared' => true])->assertRedirect();

        $entry = MapRouteSolarsystem::query()->sole();
        expect($entry->user_id)->toBeNull();

        $this->put(route('map-route-solarsystems.update', $entry), ['is_pinned' => true])->assertRedirect();
        expect($entry->fresh()->is_pinned)->toBeTrue();

        $this->delete(route('map-route-solarsystems.destroy', $entry))->assertRedirect();
        expect(MapRouteSolarsystem::query()->count())->toBe(0);
    });
});

describe('mixed watchlists', function () {
    it('returns a system on both lists twice, flagged per role', function (Permission $permission, bool $canEditShared) {
        $user = watchlistUser($this->map, $permission);
        $shared = MapRouteSolarsystem::factory()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);
        $personal = MapRouteSolarsystem::factory()->personal($user)->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);
        actingAs($user);

        $navigation = watchlistNavigation($this->map);

        expect($navigation['can_add_personal'])->toBeTrue()
            ->and($navigation['can_add_shared'])->toBe($canEditShared)
            ->and(collect($navigation['destinations'])->keyBy('id')->map(fn (array $destination): array => Arr::only($destination, ['is_personal', 'can_edit']))->all())->toBe([
                $shared->id => ['is_personal' => false, 'can_edit' => $canEditShared],
                $personal->id => ['is_personal' => true, 'can_edit' => true],
            ]);
    })->with([
        'viewer' => [Permission::Viewer, false],
        'member' => [Permission::Member, false],
        'manager' => [Permission::Manager, true],
    ]);

    it('scopes the map relation to shared rows', function () {
        $shared = MapRouteSolarsystem::factory()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);
        MapRouteSolarsystem::factory()->personal()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);

        expect($this->map->mapRouteSolarsystems()->pluck('id')->all())->toBe([$shared->id]);

        $this->map->mapRouteSolarsystems()->delete();

        expect(MapRouteSolarsystem::query()->count())->toBe(1);
    });
});

describe('cascades', function () {
    it('removes personal entries with their user', function () {
        $user = User::factory()->create();
        MapRouteSolarsystem::factory()->personal($user)->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);
        MapRouteSolarsystem::factory()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);

        $user->delete();

        expect(MapRouteSolarsystem::query()->sole()->user_id)->toBeNull();
    });

    it('removes every entry with the map', function () {
        MapRouteSolarsystem::factory()->personal()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);
        MapRouteSolarsystem::factory()->create(['map_id' => $this->map->id, 'solarsystem_id' => $this->solarsystemId]);

        $this->map->delete();

        expect(MapRouteSolarsystem::query()->count())->toBe(0);
    });
});
