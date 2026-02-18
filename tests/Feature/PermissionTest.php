<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;
use Illuminate\Support\Str;

use function Pest\Laravel\actingAs;

// --- Permission Enum Tests ---

it('has correct permission levels', function () {
    expect(Permission::Viewer->level())->toBe(1)
        ->and(Permission::Member->level())->toBe(2)
        ->and(Permission::Manager->level())->toBe(3);
});

it('can compare permission levels with isAtLeast', function () {
    expect(Permission::Viewer->isAtLeast(Permission::Viewer))->toBeTrue()
        ->and(Permission::Viewer->isAtLeast(Permission::Member))->toBeFalse()
        ->and(Permission::Viewer->isAtLeast(Permission::Manager))->toBeFalse()
        ->and(Permission::Member->isAtLeast(Permission::Viewer))->toBeTrue()
        ->and(Permission::Member->isAtLeast(Permission::Member))->toBeTrue()
        ->and(Permission::Member->isAtLeast(Permission::Manager))->toBeFalse()
        ->and(Permission::Manager->isAtLeast(Permission::Viewer))->toBeTrue()
        ->and(Permission::Manager->isAtLeast(Permission::Member))->toBeTrue()
        ->and(Permission::Manager->isAtLeast(Permission::Manager))->toBeTrue();
});

// --- Policy Tests: Viewer ---

it('allows a viewer to view the map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Viewer])->for($map)))->create();

    actingAs($user);

    expect($user->can('view', $map))->toBeTrue();
});

it('prevents a viewer from updating the map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Viewer])->for($map)))->create();

    actingAs($user);

    expect($user->can('update', $map))->toBeFalse();
});

it('prevents a viewer from viewing characters', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Viewer])->for($map)))->create();

    actingAs($user);

    expect($user->can('viewCharacters', $map))->toBeFalse();
});

it('prevents a viewer from managing access', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Viewer])->for($map)))->create();

    actingAs($user);

    expect($user->can('manageAccess', $map))->toBeFalse();
});

// --- Policy Tests: Member ---

it('allows a member to view the map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))->create();

    actingAs($user);

    expect($user->can('view', $map))->toBeTrue();
});

it('allows a member to update the map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))->create();

    actingAs($user);

    expect($user->can('update', $map))->toBeTrue();
});

it('allows a member to view characters', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))->create();

    actingAs($user);

    expect($user->can('viewCharacters', $map))->toBeTrue();
});

it('prevents a member from managing access', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))->create();

    actingAs($user);

    expect($user->can('manageAccess', $map))->toBeFalse();
});

// --- Policy Tests: Manager ---

it('allows a manager to view the map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Manager])->for($map)))->create();

    actingAs($user);

    expect($user->can('view', $map))->toBeTrue();
});

it('allows a manager to update the map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Manager])->for($map)))->create();

    actingAs($user);

    expect($user->can('update', $map))->toBeTrue();
});

it('allows a manager to manage access', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Manager])->for($map)))->create();

    actingAs($user);

    expect($user->can('manageAccess', $map))->toBeTrue();
});

it('prevents a manager from deleting the map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Manager])->for($map)))->create();

    actingAs($user);

    expect($user->can('delete', $map))->toBeFalse();
});

// --- Policy Tests: Owner ---

it('allows an owner to delete the map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    actingAs($user);

    expect($user->can('delete', $map))->toBeTrue();
});

// --- getUserPermission: returns highest tier ---

it('returns the highest permission when a user has multiple access entries', function () {
    $map = Map::factory()->create();
    $character = Character::factory()->create();

    // Grant viewer via character
    MapAccess::factory(['permission' => Permission::Viewer])->for($map)->for($character, 'accessible')->create();
    // Grant manager via corporation
    MapAccess::factory(['permission' => Permission::Manager, 'accessible_id' => $character->corporation_id, 'accessible_type' => 'App\\Models\\Corporation'])->for($map)->create();

    $user = $character->user;
    $user->load('characters.corporation', 'characters.alliance');

    expect($map->getUserPermission($user))->toBe(Permission::Manager);
});

// --- Public Map Tests ---

it('allows an anonymous user to view a public map', function () {
    $map = Map::factory()->create(['is_public' => true]);

    expect(app(App\Policies\MapPolicy::class)->view(null, $map))->toBeTrue();
});

it('prevents an anonymous user from viewing a private map', function () {
    $map = Map::factory()->create(['is_public' => false]);

    expect(app(App\Policies\MapPolicy::class)->view(null, $map))->toBeFalse();
});

it('prevents an anonymous user from updating any map', function () {
    $map = Map::factory()->create(['is_public' => true]);

    expect(app(App\Policies\MapPolicy::class)->update(null, $map))->toBeFalse();
});

it('prevents an anonymous user from viewing characters on a public map', function () {
    $map = Map::factory()->create(['is_public' => true]);

    expect(app(App\Policies\MapPolicy::class)->viewCharacters(null, $map))->toBeFalse();
});

it('prevents an anonymous user from managing access on a public map', function () {
    $map = Map::factory()->create(['is_public' => true]);

    expect(app(App\Policies\MapPolicy::class)->manageAccess(null, $map))->toBeFalse();
});

// --- Share Token Tests ---

it('resolves a map from a valid share token', function () {
    $token = Str::uuid()->toString();
    $map = Map::factory()->create(['share_token' => $token]);

    $resolved = Map::query()->where('share_token', $token)->first();

    expect($resolved)->not->toBeNull()
        ->and($resolved->id)->toBe($map->id);
});

it('returns null for an invalid share token', function () {
    $resolved = Map::query()->where('share_token', 'nonexistent-token')->first();

    expect($resolved)->toBeNull();
});

it('reports map as publicly accessible when public', function () {
    $map = Map::factory()->create(['is_public' => true]);

    expect($map->isPubliclyAccessible())->toBeTrue();
});

it('reports map as publicly accessible when share token exists', function () {
    $map = Map::factory()->create(['share_token' => Str::uuid()->toString()]);

    expect($map->isPubliclyAccessible())->toBeTrue();
});

it('reports map as not publicly accessible when private and no share token', function () {
    $map = Map::factory()->create(['is_public' => false, 'share_token' => null]);

    expect($map->isPubliclyAccessible())->toBeFalse();
});

// --- No access user ---

it('prevents a user without access from viewing a private map', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory())->create();

    actingAs($user);

    expect($user->can('view', $map))->toBeFalse();
});
