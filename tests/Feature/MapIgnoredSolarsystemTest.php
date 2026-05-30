<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapIgnoredSolarsystem;
use App\Models\User;
use Illuminate\Support\Facades\DB;

use function Pest\Laravel\actingAs;

function userWithMapPermission(Map $map, Permission $permission): User
{
    return User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();
}

function createIgnoredTestSolarsystem(int $solarsystemId = 30000042): int
{
    DB::table('regions')->insertOrIgnore(['id' => 10000042, 'name' => 'Ignored Region', 'type' => 'normal']);
    DB::table('constellations')->insertOrIgnore(['id' => 20000042, 'name' => 'Ignored Constellation', 'region_id' => 10000042, 'type' => 'normal']);
    DB::table('solarsystems')->insertOrIgnore([
        'id' => $solarsystemId,
        'name' => 'Ignored System '.$solarsystemId,
        'constellation_id' => 20000042,
        'region_id' => 10000042,
        'security' => 0.5,
        'pos_x' => 0,
        'pos_y' => 0,
        'pos_z' => 0,
        'type' => 'normal',
    ]);

    return $solarsystemId;
}

it('lets a member ignore a solar system', function () {
    $map = Map::factory()->create();
    $solarsystemId = createIgnoredTestSolarsystem();
    actingAs(userWithMapPermission($map, Permission::Member));

    $this->post(route('map-ignored-solarsystems.store'), [
        'map_id' => $map->id,
        'solarsystem_id' => $solarsystemId,
    ])->assertRedirect();

    expect(MapIgnoredSolarsystem::query()
        ->where('map_id', $map->id)
        ->where('solarsystem_id', $solarsystemId)
        ->exists()
    )->toBeTrue();
});

it('is idempotent when ignoring the same system twice', function () {
    $map = Map::factory()->create();
    $solarsystemId = createIgnoredTestSolarsystem();
    actingAs(userWithMapPermission($map, Permission::Member));

    $payload = ['map_id' => $map->id, 'solarsystem_id' => $solarsystemId];

    $this->post(route('map-ignored-solarsystems.store'), $payload)->assertRedirect();
    $this->post(route('map-ignored-solarsystems.store'), $payload)->assertRedirect();

    expect(MapIgnoredSolarsystem::query()->where('map_id', $map->id)->count())->toBe(1);
});

it('lets a member unignore a solar system', function () {
    $map = Map::factory()->create();
    $solarsystemId = createIgnoredTestSolarsystem();
    $ignored = MapIgnoredSolarsystem::factory()->for($map)->create(['solarsystem_id' => $solarsystemId]);
    actingAs(userWithMapPermission($map, Permission::Member));

    $this->delete(route('map-ignored-solarsystems.destroy', [
        'map' => $map,
        'solarsystem_id' => $ignored->solarsystem_id,
    ]))->assertRedirect();

    expect(MapIgnoredSolarsystem::query()->whereKey($ignored->id)->exists())->toBeFalse();
});

it('lets a member clear the entire ignore list', function () {
    $map = Map::factory()->create();
    $ids = [createIgnoredTestSolarsystem(30000042), createIgnoredTestSolarsystem(30000043), createIgnoredTestSolarsystem(30000044)];
    foreach ($ids as $id) {
        MapIgnoredSolarsystem::factory()->for($map)->create(['solarsystem_id' => $id]);
    }
    actingAs(userWithMapPermission($map, Permission::Member));

    $this->delete(route('map-ignored-solarsystems.destroy-all', $map))->assertRedirect();

    expect(MapIgnoredSolarsystem::query()->where('map_id', $map->id)->count())->toBe(0);
});

it('forbids a viewer from ignoring a system', function () {
    $map = Map::factory()->create();
    $solarsystemId = createIgnoredTestSolarsystem();
    actingAs(userWithMapPermission($map, Permission::Viewer));

    $this->post(route('map-ignored-solarsystems.store'), [
        'map_id' => $map->id,
        'solarsystem_id' => $solarsystemId,
    ])->assertForbidden();

    expect(MapIgnoredSolarsystem::query()->where('map_id', $map->id)->count())->toBe(0);
});

it('forbids a viewer from clearing the ignore list', function () {
    $map = Map::factory()->create();
    $solarsystemId = createIgnoredTestSolarsystem();
    MapIgnoredSolarsystem::factory()->for($map)->create(['solarsystem_id' => $solarsystemId]);
    actingAs(userWithMapPermission($map, Permission::Viewer));

    $this->delete(route('map-ignored-solarsystems.destroy-all', $map))->assertForbidden();

    expect(MapIgnoredSolarsystem::query()->where('map_id', $map->id)->count())->toBe(1);
});

it('exposes the ignored systems on the map page', function () {
    $map = Map::factory()->create();
    $solarsystemId = createIgnoredTestSolarsystem();
    MapIgnoredSolarsystem::factory()->for($map)->create(['solarsystem_id' => $solarsystemId]);
    $user = User::factory()->ownsMap($map)->create();
    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    actingAs($user);

    $this->withoutExceptionHandling()
        ->get(route('maps.show', $map))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page
            ->where('map_ignored_systems', [$solarsystemId])
            ->etc()
        );
});
