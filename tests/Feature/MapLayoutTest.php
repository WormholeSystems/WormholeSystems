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
    return User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();
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
