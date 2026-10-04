<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Events\Maps\MapMetadataUpdatedEvent;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;
use Illuminate\Support\Facades\Event;

use function Pest\Laravel\actingAs;

function rallyPointUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

beforeEach(function () {
    makeSolarsystem(31000001);
    makeSolarsystem(31000002);
    makeSolarsystem(31000003);
});

it('starts a new map without rally points', function () {
    expect(Map::factory()->create()->rally_solarsystem_ids)->toBe([])
        ->and(Map::factory()->create()->fresh()->rally_solarsystem_ids)->toBe([]);
});

it('lets a member set a rally point and broadcasts the change', function () {
    Event::fake([MapMetadataUpdatedEvent::class]);
    $map = Map::factory()->create();

    actingAs(rallyPointUser($map, Permission::Member))
        ->post("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 31000001])
        ->assertRedirect();

    expect($map->fresh()->rally_solarsystem_ids)->toBe([31000001]);
    Event::assertDispatched(MapMetadataUpdatedEvent::class);
});

it('keeps two rally points at the same time', function () {
    $map = Map::factory()->create(['rally_solarsystem_ids' => [31000001]]);

    actingAs(rallyPointUser($map, Permission::Member))
        ->post("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 31000002])
        ->assertRedirect();

    expect($map->fresh()->rally_solarsystem_ids)->toBe([31000001, 31000002]);
});

it('replaces the oldest rally point once the cap is reached', function () {
    $map = Map::factory()->create(['rally_solarsystem_ids' => [31000001, 31000002]]);

    actingAs(rallyPointUser($map, Permission::Member))
        ->post("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 31000003])
        ->assertRedirect();

    expect($map->fresh()->rally_solarsystem_ids)->toBe([31000002, 31000003]);
});

it('respects a configured rally point cap', function () {
    config(['map.max_rally_points' => 3]);
    $map = Map::factory()->create(['rally_solarsystem_ids' => [31000001, 31000002]]);

    actingAs(rallyPointUser($map, Permission::Member))
        ->post("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 31000003])
        ->assertRedirect();

    expect($map->fresh()->rally_solarsystem_ids)->toBe([31000001, 31000002, 31000003]);
});

it('does not duplicate an existing rally point', function () {
    $map = Map::factory()->create(['rally_solarsystem_ids' => [31000001, 31000002]]);

    actingAs(rallyPointUser($map, Permission::Member))
        ->post("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 31000001])
        ->assertRedirect();

    expect($map->fresh()->rally_solarsystem_ids)->toBe([31000002, 31000001]);
});

it('clears a single rally point and keeps the other', function () {
    Event::fake([MapMetadataUpdatedEvent::class]);
    $map = Map::factory()->create(['rally_solarsystem_ids' => [31000001, 31000002]]);

    actingAs(rallyPointUser($map, Permission::Member))
        ->delete("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 31000001])
        ->assertRedirect();

    expect($map->fresh()->rally_solarsystem_ids)->toBe([31000002]);
    Event::assertDispatched(MapMetadataUpdatedEvent::class);
});

it('ignores clearing a system that is not a rally point', function () {
    $map = Map::factory()->create(['rally_solarsystem_ids' => [31000001]]);

    actingAs(rallyPointUser($map, Permission::Member))
        ->delete("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 31000002])
        ->assertRedirect();

    expect($map->fresh()->rally_solarsystem_ids)->toBe([31000001]);
});

it('rejects an unknown solarsystem', function () {
    $map = Map::factory()->create();

    actingAs(rallyPointUser($map, Permission::Member))
        ->post("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 99999999])
        ->assertSessionHasErrors('solarsystem_id');

    expect($map->fresh()->rally_solarsystem_ids)->toBe([]);
});

it('requires a solarsystem', function (string $method) {
    $map = Map::factory()->create(['rally_solarsystem_ids' => [31000001]]);

    actingAs(rallyPointUser($map, Permission::Member))
        ->{$method}("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => null])
        ->assertSessionHasErrors('solarsystem_id');

    expect($map->fresh()->rally_solarsystem_ids)->toBe([31000001]);
})->with(['post', 'delete']);

it('forbids a viewer from changing rally points', function (string $method) {
    $map = Map::factory()->create(['rally_solarsystem_ids' => [31000001]]);

    actingAs(rallyPointUser($map, Permission::Viewer))
        ->{$method}("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 31000002])
        ->assertForbidden();

    expect($map->fresh()->rally_solarsystem_ids)->toBe([31000001]);
})->with(['post', 'delete']);

it('exposes the rally points in the map metadata', function () {
    Event::fake([MapMetadataUpdatedEvent::class]);
    $map = Map::factory()->create();

    actingAs(rallyPointUser($map, Permission::Member))
        ->post("/maps/{$map->slug}/settings/rally-point", ['solarsystem_id' => 31000001]);

    Event::assertDispatched(MapMetadataUpdatedEvent::class, fn (MapMetadataUpdatedEvent $event): bool => $event->broadcastWith()['map']['rally_solarsystem_ids'] === [31000001]);
});
