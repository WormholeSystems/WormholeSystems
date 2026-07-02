<?php

declare(strict_types=1);

use App\Jobs\Characters\UpdateCharacterLocation;
use App\Models\Category;
use App\Models\Character;
use App\Models\CharacterStatus;
use App\Models\Group;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapConnectionJump;
use App\Models\MapUserSetting;
use App\Models\Type;
use NicolasKion\Esi\DTO\EsiError;
use NicolasKion\Esi\DTO\EsiResult;
use NicolasKion\Esi\DTO\Location;
use NicolasKion\Esi\DTO\Ship;
use NicolasKion\Esi\Esi;

const LOCATION_SHIP_TYPE_ID = 73791;

const LOCATION_SHIP_MASS = 130_000_000;

/**
 * @return array{character: Character, status: CharacterStatus, connection: MapConnection}
 */
function createTrackedCharacterOnConnection(int $from_solarsystem_id, int $to_solarsystem_id): array
{
    $character = Character::factory()->create();

    $map = Map::factory()->create();
    MapUserSetting::query()->create([
        'map_id' => $map->id,
        'user_id' => $character->user_id,
        'tracking_allowed' => true,
        'is_tracking' => true,
    ]);

    $origin = placeMapSolarsystem($map, $from_solarsystem_id);
    $target = placeMapSolarsystem($map, $to_solarsystem_id, 300, 300);

    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $origin->id,
        'to_map_solarsystem_id' => $target->id,
    ]);

    Category::query()->firstOrCreate(['id' => 6], ['name' => 'Ship']);
    Group::query()->firstOrCreate(['id' => 27], ['name' => 'Battleship', 'category_id' => 6]);
    Type::query()->firstOrCreate(
        ['id' => LOCATION_SHIP_TYPE_ID],
        ['name' => 'Nestor', 'group_id' => 27, 'mass' => LOCATION_SHIP_MASS],
    );

    $status = CharacterStatus::query()->create([
        'character_id' => $character->id,
        'solarsystem_id' => $from_solarsystem_id,
        'is_online' => true,
    ]);

    return ['character' => $character, 'status' => $status, 'connection' => $connection];
}

function fakeEsiLocation(?Location $location, ?Ship $ship = null): void
{
    $esi = test()->mock(Esi::class);

    if (! $location instanceof Location) {
        $esi->shouldReceive('getLocation')->andReturn(new EsiResult(error: new EsiError(500, 'unavailable')));

        return;
    }

    $esi->shouldReceive('getLocation')->andReturn(new EsiResult(data: $location));
    $esi->shouldReceive('getShip')->andReturn(new EsiResult(
        data: $ship ?? new Ship(ship_type_id: LOCATION_SHIP_TYPE_ID, ship_item_id: 9001, ship_name: 'Blackbetty'),
    ));
}

it('records a connection jump when a tracked character moves between connected systems', function () {
    ['status' => $status, 'connection' => $connection] = createTrackedCharacterOnConnection(31000201, 31000202);

    fakeEsiLocation(new Location(solar_system_id: 31000202));

    UpdateCharacterLocation::dispatchSync($status->id);

    expect($status->fresh()->solarsystem_id)->toBe(31000202);

    $jump = MapConnectionJump::query()->sole();
    expect($jump->map_connection_id)->toBe($connection->id)
        ->and($jump->ship_type_id)->toBe(LOCATION_SHIP_TYPE_ID)
        ->and($jump->ship_name)->toBe('Blackbetty')
        ->and($jump->mass)->toBe(LOCATION_SHIP_MASS);
});

it('does not record a jump when the character stays in the same system', function () {
    ['status' => $status] = createTrackedCharacterOnConnection(31000203, 31000204);

    fakeEsiLocation(new Location(solar_system_id: 31000203, structure_id: 1035000000000));

    UpdateCharacterLocation::dispatchSync($status->id);

    expect($status->fresh()->structure_id)->toBe(1035000000000)
        ->and(MapConnectionJump::query()->count())->toBe(0);
});

it('records nothing when the location request fails', function () {
    ['status' => $status] = createTrackedCharacterOnConnection(31000205, 31000206);

    fakeEsiLocation(null);

    UpdateCharacterLocation::dispatchSync($status->id);

    expect($status->fresh()->solarsystem_id)->toBe(31000205)
        ->and(MapConnectionJump::query()->count())->toBe(0);
});
