<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;
use App\Scopes\CharacterHasMapAccess;

use function Pest\Laravel\actingAs;
use function Pest\Laravel\artisan;

// --- Expired access denies map view ---

it('denies map view when access has expired', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(
        Character::factory()->has(
            MapAccess::factory(['permission' => Permission::Viewer])->expired()->for($map)
        )
    )->create();

    actingAs($user);

    expect($user->can('view', $map))->toBeFalse();
});

// --- Non-expired access allows map view ---

it('allows map view when access has not expired', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(
        Character::factory()->has(
            MapAccess::factory(['permission' => Permission::Viewer])->expiresIn(24)->for($map)
        )
    )->create();

    actingAs($user);

    expect($user->can('view', $map))->toBeTrue();
});

// --- getUserPermission returns null for expired access ---

it('returns null for getUserPermission when access has expired', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(
        Character::factory()->has(
            MapAccess::factory(['permission' => Permission::Manager])->expired()->for($map)
        )
    )->create();

    expect($map->getUserPermission($user))->toBeNull();
});

// --- Highest non-expired permission wins with mixed entries ---

it('returns highest non-expired permission when mixed entries exist', function () {
    $map = Map::factory()->create();
    $character = Character::factory()->create();

    // Expired manager via character
    MapAccess::factory(['permission' => Permission::Manager])->expired()->for($map)->for($character, 'accessible')->create();

    // Non-expired viewer via corporation
    MapAccess::factory([
        'permission' => Permission::Viewer,
        'accessible_id' => $character->corporation_id,
        'accessible_type' => 'App\\Models\\Corporation',
    ])->expiresIn(24)->for($map)->create();

    $user = $character->user;
    $user->load('characters.corporation', 'characters.alliance');

    expect($map->getUserPermission($user))->toBe(Permission::Viewer);
});

// --- CharacterHasMapAccess scope excludes expired entries ---

it('excludes expired entries from CharacterHasMapAccess scope', function () {
    $map = Map::factory()->create();
    $character = Character::factory()->create();

    MapAccess::factory(['permission' => Permission::Member])->expired()->for($map)->for($character, 'accessible')->create();

    $scope = new CharacterHasMapAccess($map);
    $result = Character::query()->tap($scope)->where('id', $character->id)->exists();

    expect($result)->toBeFalse();
});

it('includes non-expired entries in CharacterHasMapAccess scope', function () {
    $map = Map::factory()->create();
    $character = Character::factory()->create();

    MapAccess::factory(['permission' => Permission::Member])->expiresIn(24)->for($map)->for($character, 'accessible')->create();

    $scope = new CharacterHasMapAccess($map);
    $result = Character::query()->tap($scope)->where('id', $character->id)->exists();

    expect($result)->toBeTrue();
});

// --- Purge command deletes expired, keeps active ---

it('purges expired map access entries and keeps active ones', function () {
    $map = Map::factory()->create();

    $expiredAccess = MapAccess::factory(['permission' => Permission::Viewer])
        ->expired()
        ->for($map)
        ->for(Character::factory(), 'accessible')
        ->create();

    $activeAccess = MapAccess::factory(['permission' => Permission::Member])
        ->expiresIn(24)
        ->for($map)
        ->for(Character::factory(), 'accessible')
        ->create();

    $permanentAccess = MapAccess::factory(['permission' => Permission::Manager])
        ->for($map)
        ->for(Character::factory(), 'accessible')
        ->create();

    artisan('app:purge-expired-map-access')->assertSuccessful();

    expect(MapAccess::find($expiredAccess->id))->toBeNull()
        ->and(MapAccess::find($activeAccess->id))->not->toBeNull()
        ->and(MapAccess::find($permanentAccess->id))->not->toBeNull();
});

// --- Validation rejects past expires_at ---

it('rejects a past expires_at value', function () {
    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    actingAs($user);

    $response = $this->post(route('maps.settings.access.store', $map), [
        'entity_id' => Character::factory()->create()->id,
        'entity_type' => 'character',
        'permission' => 'viewer',
        'expires_at' => now()->subHour()->toISOString(),
    ]);

    $response->assertSessionHasErrors('expires_at');
});

// --- Validation accepts null expires_at ---

it('accepts null expires_at value', function () {
    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    actingAs($user);

    $character = Character::factory()->create();

    $response = $this->post(route('maps.settings.access.store', $map), [
        'entity_id' => $character->id,
        'entity_type' => 'character',
        'permission' => 'viewer',
        'expires_at' => null,
    ]);

    $response->assertSessionHasNoErrors();
});

// --- Null expires_at means permanent access ---

it('treats null expires_at as permanent access', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(
        Character::factory()->has(
            MapAccess::factory(['permission' => Permission::Viewer, 'expires_at' => null])->for($map)
        )
    )->create();

    actingAs($user);

    expect($user->can('view', $map))->toBeTrue();
});

// --- isExpired accessor ---

it('correctly identifies expired access via isExpired', function () {
    $expired = MapAccess::factory(['permission' => Permission::Viewer])
        ->expired()
        ->for(Map::factory())
        ->for(Character::factory(), 'accessible')
        ->create();

    $active = MapAccess::factory(['permission' => Permission::Viewer])
        ->expiresIn(24)
        ->for(Map::factory())
        ->for(Character::factory(), 'accessible')
        ->create();

    $permanent = MapAccess::factory(['permission' => Permission::Viewer])
        ->for(Map::factory())
        ->for(Character::factory(), 'accessible')
        ->create();

    expect($expired->isExpired())->toBeTrue()
        ->and($active->isExpired())->toBeFalse()
        ->and($permanent->isExpired())->toBeFalse();
});
