<?php

declare(strict_types=1);

use App\Enums\LifetimeStatus;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\Signature;
use App\Models\User;

use function Pest\Laravel\actingAs;

function lifetimeMember(Map $map): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

function lifetimeConnection(Map $map, LifetimeStatus $lifetime = LifetimeStatus::Healthy): MapConnection
{
    $from = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => makeSolarsystem(30009701)]);
    $to = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => makeSolarsystem(30009702)]);

    return MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        'lifetime' => $lifetime,
    ]);
}

it('lets a member mark a connection and its signature as expired', function () {
    $map = Map::factory()->create();
    $connection = lifetimeConnection($map, LifetimeStatus::Critical);
    $signature = Signature::factory()->create([
        'map_solarsystem_id' => $connection->from_map_solarsystem_id,
        'map_connection_id' => $connection->id,
        'lifetime' => LifetimeStatus::Critical,
    ]);

    actingAs(lifetimeMember($map))
        ->put("/map-connections/{$connection->id}", ['lifetime' => 'expired'])
        ->assertRedirect()
        ->assertSessionHasNoErrors();

    expect($connection->fresh()->lifetime)->toBe(LifetimeStatus::Expired)
        ->and($connection->fresh()->lifetime_updated_at)->not->toBeNull()
        ->and($signature->fresh()->lifetime)->toBe(LifetimeStatus::Expired);
});

it('keeps an expired connection expired when it is resynced from its signatures', function () {
    $map = Map::factory()->create();
    $connection = lifetimeConnection($map, LifetimeStatus::Expired);
    Signature::factory()->create([
        'map_solarsystem_id' => $connection->from_map_solarsystem_id,
        'map_connection_id' => $connection->id,
        'lifetime' => LifetimeStatus::Critical,
    ]);

    actingAs(lifetimeMember($map))
        ->put("/map-connections/{$connection->id}", ['preserve_mass' => true])
        ->assertRedirect();

    expect($connection->fresh()->lifetime)->toBe(LifetimeStatus::Expired);
});

it('rejects an unknown lifetime status', function () {
    $map = Map::factory()->create();
    $connection = lifetimeConnection($map);

    actingAs(lifetimeMember($map))
        ->put("/map-connections/{$connection->id}", ['lifetime' => 'collapsed'])
        ->assertSessionHasErrors('lifetime');

    expect($connection->fresh()->lifetime)->toBe(LifetimeStatus::Healthy);
});
