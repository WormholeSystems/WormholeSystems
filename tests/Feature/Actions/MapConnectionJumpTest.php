<?php

declare(strict_types=1);

use App\Actions\MapConnections\RecordMapConnectionJumpAction;
use App\Actions\Tracking\StoreTrackingAction;
use App\Console\Commands\MapConnections\PruneUnclaimedConnectionJumpsCommand;
use App\Data\TrackingData;
use App\Enums\ConnectionType;
use App\Events\MapConnections\MapConnectionsUpsertedEvent;
use App\Models\Category;
use App\Models\Character;
use App\Models\Group;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapConnectionJump;
use App\Models\MapUserSetting;
use App\Models\Type;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Event;

const JUMP_SHIP_TYPE_ID = 73790;

const JUMP_SHIP_MASS = 130_000_000;

function createTrackedCharacter(): Character
{
    return Character::factory()->create();
}

function createTrackedMap(Character $character, bool $tracking_allowed = true, bool $is_tracking = true): Map
{
    $map = Map::factory()->create();

    MapUserSetting::query()->create([
        'map_id' => $map->id,
        'user_id' => $character->user_id,
        'tracking_allowed' => $tracking_allowed,
        'is_tracking' => $is_tracking,
    ]);

    return $map;
}

function createJumpShipType(): Type
{
    Category::query()->firstOrCreate(['id' => 6], ['name' => 'Ship']);
    Group::query()->firstOrCreate(['id' => 27], ['name' => 'Battleship', 'category_id' => 6]);

    return Type::query()->firstOrCreate(
        ['id' => JUMP_SHIP_TYPE_ID],
        ['name' => 'Praxis', 'group_id' => 27, 'mass' => JUMP_SHIP_MASS],
    );
}

/**
 * @return array{map: Map, connection: MapConnection, from: int, to: int}
 */
function createTrackedMapWithConnection(Character $character, int $from_solarsystem_id, int $to_solarsystem_id): array
{
    $map = createTrackedMap($character);
    $origin = placeMapSolarsystem($map, $from_solarsystem_id);
    $target = placeMapSolarsystem($map, $to_solarsystem_id, 300, 300);

    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $origin->id,
        'to_map_solarsystem_id' => $target->id,
    ]);

    return ['map' => $map, 'connection' => $connection, 'from' => $from_solarsystem_id, 'to' => $to_solarsystem_id];
}

function linkSolarsystemsByStargate(int $from_solarsystem_id, int $to_solarsystem_id): void
{
    Group::query()->firstOrCreate(['id' => 10], ['name' => 'Stargate', 'category_id' => 6]);
    Type::query()->firstOrCreate(['id' => 16], ['name' => 'Stargate', 'group_id' => 10]);

    $stargates = [
        ['id' => $from_solarsystem_id * 100, 'solarsystem_id' => $from_solarsystem_id],
        ['id' => $to_solarsystem_id * 100, 'solarsystem_id' => $to_solarsystem_id],
    ];

    foreach ($stargates as $stargate) {
        DB::table('stargates')->insertOrIgnore([
            'id' => $stargate['id'],
            'solarsystem_id' => $stargate['solarsystem_id'],
            'constellation_id' => 20009000,
            'region_id' => 10009000,
            'type_id' => 16,
        ]);
    }

    DB::table('solarsystem_connections')->insertOrIgnore([
        'from_stargate_id' => $from_solarsystem_id * 100,
        'from_solarsystem_id' => $from_solarsystem_id,
        'from_constellation_id' => 20009000,
        'from_region_id' => 10009000,
        'to_stargate_id' => $to_solarsystem_id * 100,
        'to_solarsystem_id' => $to_solarsystem_id,
        'to_constellation_id' => 20009000,
        'to_region_id' => 10009000,
    ]);
}

function recordJump(Character $character, int $from_solarsystem_id, int $to_solarsystem_id): void
{
    app(RecordMapConnectionJumpAction::class)->handle(
        (int) $character->id,
        $from_solarsystem_id,
        $to_solarsystem_id,
        JUMP_SHIP_TYPE_ID,
        'Test Ship',
    );
}

it('records a jump through an existing wormhole connection with the ship mass snapshot', function () {
    Event::fake([MapConnectionsUpsertedEvent::class]);

    $character = createTrackedCharacter();
    createJumpShipType();
    ['connection' => $connection] = createTrackedMapWithConnection($character, 31000101, 31000102);

    recordJump($character, 31000101, 31000102);

    $jump = MapConnectionJump::query()->sole();
    expect($jump->map_connection_id)->toBe($connection->id)
        ->and($jump->character_id)->toBe((int) $character->id)
        ->and($jump->ship_type_id)->toBe(JUMP_SHIP_TYPE_ID)
        ->and($jump->mass)->toBe(JUMP_SHIP_MASS);

    Event::assertDispatched(MapConnectionsUpsertedEvent::class, function (MapConnectionsUpsertedEvent $event) use ($connection): bool {
        return collect($event->broadcastWith()['map_connections'])
            ->contains(fn (array $payload): bool => $payload['id'] === $connection->id
                && $payload['jumps_mass_sum'] === JUMP_SHIP_MASS
                && $payload['jumps_count'] === 1);
    });
});

it('records a jump in the reverse direction of the connection', function () {
    $character = createTrackedCharacter();
    createJumpShipType();
    ['connection' => $connection] = createTrackedMapWithConnection($character, 31000103, 31000104);

    recordJump($character, 31000104, 31000103);

    expect(MapConnectionJump::query()->sole()->map_connection_id)->toBe($connection->id);
});

it('skips stargate-type connections', function () {
    $character = createTrackedCharacter();
    createJumpShipType();
    ['connection' => $connection] = createTrackedMapWithConnection($character, 31000105, 31000106);
    $connection->update(['type' => ConnectionType::Stargate]);

    recordJump($character, 31000105, 31000106);

    expect(MapConnectionJump::query()->count())->toBe(0);
});

it('stores a pending jump when the connection does not exist yet and broadcasts nothing', function () {
    Event::fake([MapConnectionsUpsertedEvent::class]);

    $character = createTrackedCharacter();
    createJumpShipType();
    $map = createTrackedMap($character);
    placeMapSolarsystem($map, 31000107);
    makeSolarsystem(31000108);

    recordJump($character, 31000107, 31000108);

    $jump = MapConnectionJump::query()->sole();
    expect($jump->map_connection_id)->toBeNull()
        ->and($jump->map_id)->toBe($map->id)
        ->and($jump->mass)->toBe(JUMP_SHIP_MASS);

    Event::assertNotDispatched(MapConnectionsUpsertedEvent::class);
});

it('stores nothing when the origin system is not on the map', function () {
    $character = createTrackedCharacter();
    createJumpShipType();
    createTrackedMap($character);
    makeSolarsystem(31000109);
    makeSolarsystem(31000110);

    recordJump($character, 31000109, 31000110);

    expect(MapConnectionJump::query()->count())->toBe(0);
});

it('stores nothing for k-space systems linked by stargates', function () {
    $character = createTrackedCharacter();
    createJumpShipType();
    $map = createTrackedMap($character);
    makeSolarsystem(31000111, type: 'eve');
    makeSolarsystem(31000112, type: 'eve');
    placeMapSolarsystem($map, 31000111);
    linkSolarsystemsByStargate(31000111, 31000112);

    recordJump($character, 31000111, 31000112);

    expect(MapConnectionJump::query()->count())->toBe(0);
});

it('fans out to every map tracking the character but skips maps without tracking consent', function () {
    $character = createTrackedCharacter();
    createJumpShipType();
    ['connection' => $first_connection] = createTrackedMapWithConnection($character, 31000113, 31000114);

    $second_map = createTrackedMap($character);
    $second_origin = placeMapSolarsystem($second_map, 31000113);
    $second_target = placeMapSolarsystem($second_map, 31000114, 300, 300);
    $second_connection = MapConnection::factory()->create([
        'map_id' => $second_map->id,
        'from_map_solarsystem_id' => $second_origin->id,
        'to_map_solarsystem_id' => $second_target->id,
    ]);

    $untracked_map = createTrackedMap($character, is_tracking: false);
    $untracked_origin = placeMapSolarsystem($untracked_map, 31000113);
    $untracked_target = placeMapSolarsystem($untracked_map, 31000114, 300, 300);
    MapConnection::factory()->create([
        'map_id' => $untracked_map->id,
        'from_map_solarsystem_id' => $untracked_origin->id,
        'to_map_solarsystem_id' => $untracked_target->id,
    ]);

    recordJump($character, 31000113, 31000114);

    expect(MapConnectionJump::query()->pluck('map_connection_id')->sort()->values()->all())
        ->toBe(collect([$first_connection->id, $second_connection->id])->sort()->values()->all());
});

it('claims a pending jump when the tracked connection is created and broadcasts the jump summary', function () {
    Event::fake([MapConnectionsUpsertedEvent::class]);

    $character = createTrackedCharacter();
    createJumpShipType();
    $map = createTrackedMap($character);
    $origin = placeMapSolarsystem($map, 31000115);
    makeSolarsystem(31000116);

    recordJump($character, 31000115, 31000116);

    expect(MapConnectionJump::query()->sole()->map_connection_id)->toBeNull();

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => 31000116,
    ]));

    $connection = MapConnection::query()->where('map_id', $map->id)->sole();
    expect(MapConnectionJump::query()->sole()->map_connection_id)->toBe($connection->id);

    Event::assertDispatched(MapConnectionsUpsertedEvent::class, function (MapConnectionsUpsertedEvent $event) use ($connection): bool {
        return collect($event->broadcastWith()['map_connections'])
            ->contains(fn (array $payload): bool => $payload['id'] === $connection->id
                && $payload['jumps_mass_sum'] === JUMP_SHIP_MASS
                && $payload['jumps_count'] === 1);
    });
});

it('does not claim stale pendings or pendings of other system pairs', function () {
    $character = createTrackedCharacter();
    createJumpShipType();
    $map = createTrackedMap($character);
    $origin = placeMapSolarsystem($map, 31000117);
    makeSolarsystem(31000118);
    makeSolarsystem(31000119);

    recordJump($character, 31000117, 31000118);
    $stale = MapConnectionJump::query()->sole();
    $stale->update(['created_at' => now()->subMinutes(5)]);

    recordJump($character, 31000117, 31000119);
    $other_pair = MapConnectionJump::query()->whereKeyNot($stale->id)->sole();

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => 31000118,
    ]));

    expect($stale->fresh()->map_connection_id)->toBeNull()
        ->and($other_pair->fresh()->map_connection_id)->toBeNull();
});

it('prunes only stale unclaimed jumps', function () {
    $character = createTrackedCharacter();
    createJumpShipType();
    ['connection' => $connection] = createTrackedMapWithConnection($character, 31000120, 31000121);
    $map = $connection->map;

    $claimed = MapConnectionJump::factory()->create([
        'map_id' => $map->id,
        'map_connection_id' => $connection->id,
        'character_id' => $character->id,
        'from_solarsystem_id' => 31000120,
        'to_solarsystem_id' => 31000121,
        'created_at' => now()->subHour(),
    ]);
    $stale_pending = MapConnectionJump::factory()->pending()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'from_solarsystem_id' => 31000120,
        'to_solarsystem_id' => 31000121,
        'created_at' => now()->subHour(),
    ]);
    $fresh_pending = MapConnectionJump::factory()->pending()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'from_solarsystem_id' => 31000120,
        'to_solarsystem_id' => 31000121,
    ]);

    $this->artisan(PruneUnclaimedConnectionJumpsCommand::class)->assertSuccessful();

    expect(MapConnectionJump::query()->pluck('id')->sort()->values()->all())
        ->toBe(collect([$claimed->id, $fresh_pending->id])->sort()->values()->all())
        ->and(MapConnectionJump::query()->whereKey($stale_pending->id)->exists())->toBeFalse();
});

it('deletes the jump log together with its connection', function () {
    $character = createTrackedCharacter();
    createJumpShipType();
    ['connection' => $connection] = createTrackedMapWithConnection($character, 31000122, 31000123);

    recordJump($character, 31000122, 31000123);
    expect(MapConnectionJump::query()->count())->toBe(1);

    $connection->delete();

    expect(MapConnectionJump::query()->count())->toBe(0);
});
