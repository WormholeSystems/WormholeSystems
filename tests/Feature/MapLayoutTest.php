<?php

declare(strict_types=1);

use App\Enums\MapLayout;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;

use function Pest\Laravel\actingAs;

function layoutTestUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

it('defaults a new map to the manual layout', function () {
    expect(Map::factory()->create()->fresh()->layout)->toBe(MapLayout::Manual);
});

it('lets a manager switch the map layout', function () {
    $map = Map::factory()->create();

    actingAs(layoutTestUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/layout", ['layout' => 'tree'])
        ->assertRedirect();

    expect($map->fresh()->layout)->toBe(MapLayout::Tree);
});

it('forbids a member from switching the map layout', function () {
    $map = Map::factory()->create();

    actingAs(layoutTestUser($map, Permission::Member))
        ->put("/maps/{$map->slug}/layout", ['layout' => 'tree'])
        ->assertForbidden();

    expect($map->fresh()->layout)->toBe(MapLayout::Manual);
});

it('forbids a viewer from switching the map layout', function () {
    $map = Map::factory()->create();

    actingAs(layoutTestUser($map, Permission::Viewer))
        ->put("/maps/{$map->slug}/layout", ['layout' => 'tree'])
        ->assertForbidden();
});

it('rejects an invalid layout value', function () {
    $map = Map::factory()->create();

    actingAs(layoutTestUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/layout", ['layout' => 'spiral'])
        ->assertSessionHasErrors('layout');

    expect($map->fresh()->layout)->toBe(MapLayout::Manual);
});

it('lets a manager allow per-user layout override', function () {
    $map = Map::factory()->create();

    actingAs(layoutTestUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/layout", ['allow_layout_override' => true])
        ->assertRedirect();

    expect($map->fresh()->allow_layout_override)->toBeTrue();
});

it('forbids a member from allowing per-user layout override', function () {
    $map = Map::factory()->create();

    actingAs(layoutTestUser($map, Permission::Member))
        ->put("/maps/{$map->slug}/layout", ['allow_layout_override' => true])
        ->assertForbidden();

    expect($map->fresh()->allow_layout_override)->toBeFalse();
});

it('lets any member save a personal layout override', function () {
    $map = Map::factory()->create();
    $user = layoutTestUser($map, Permission::Viewer);

    actingAs($user)
        ->put("/maps/{$map->slug}/user-settings", ['layout_override' => 'tree'])
        ->assertRedirect();

    expect($user->mapUserSettings()->where('map_id', $map->id)->value('layout_override'))->toBe(MapLayout::Tree);
});

it('lets a manager enable constant width', function () {
    $map = Map::factory()->create();

    actingAs(layoutTestUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/layout", ['constant_width_enabled' => true])
        ->assertRedirect();

    expect($map->fresh()->constant_width_enabled)->toBeTrue();
});

it('forbids a member from enabling constant width', function () {
    $map = Map::factory()->create();

    actingAs(layoutTestUser($map, Permission::Member))
        ->put("/maps/{$map->slug}/layout", ['constant_width_enabled' => true])
        ->assertForbidden();

    expect($map->fresh()->constant_width_enabled)->toBeFalse();
});
