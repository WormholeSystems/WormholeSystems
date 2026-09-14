<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;

use function Pest\Laravel\actingAs;

function maintainerSettingsUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

it('defaults a new map to 1/1/1/20', function () {
    $map = Map::factory()->create()->fresh();

    expect($map->maintainer_points_created)->toBe(1)
        ->and($map->maintainer_points_updated)->toBe(1)
        ->and($map->maintainer_points_deleted)->toBe(1)
        ->and($map->maintainer_minimum_points)->toBe(20);
});

it('lets a manager update the maintainer settings', function () {
    $map = Map::factory()->create();

    actingAs(maintainerSettingsUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/settings/maintainer", [
            'maintainer_points_created' => 2,
            'maintainer_points_updated' => 3,
            'maintainer_points_deleted' => 5,
            'maintainer_minimum_points' => 50,
        ])
        ->assertRedirect()
        ->assertSessionHasNoErrors();

    $map->refresh();

    expect($map->maintainer_points_created)->toBe(2)
        ->and($map->maintainer_points_updated)->toBe(3)
        ->and($map->maintainer_points_deleted)->toBe(5)
        ->and($map->maintainer_minimum_points)->toBe(50);
});

it('forbids a member from updating the maintainer settings', function () {
    $map = Map::factory()->create();

    actingAs(maintainerSettingsUser($map, Permission::Member))
        ->put("/maps/{$map->slug}/settings/maintainer", ['maintainer_points_created' => 2])
        ->assertForbidden();

    expect($map->fresh()->maintainer_points_created)->toBe(1);
});

it('forbids a viewer from updating the maintainer settings', function () {
    $map = Map::factory()->create();

    actingAs(maintainerSettingsUser($map, Permission::Viewer))
        ->put("/maps/{$map->slug}/settings/maintainer", ['maintainer_points_created' => 2])
        ->assertForbidden();
});

it('rejects out-of-range values', function (string $field, int $value) {
    $map = Map::factory()->create();

    actingAs(maintainerSettingsUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/settings/maintainer", [$field => $value])
        ->assertSessionHasErrors($field);
})->with([
    'created below range' => ['maintainer_points_created', -1],
    'created above range' => ['maintainer_points_created', 101],
    'updated below range' => ['maintainer_points_updated', -1],
    'updated above range' => ['maintainer_points_updated', 101],
    'deleted below range' => ['maintainer_points_deleted', -1],
    'deleted above range' => ['maintainer_points_deleted', 101],
    'minimum below range' => ['maintainer_minimum_points', -1],
    'minimum above range' => ['maintainer_minimum_points', 10001],
]);
