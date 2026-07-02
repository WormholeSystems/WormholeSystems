<?php

declare(strict_types=1);

use App\Actions\MapConnections\CleanStaleMapConnectionsAction;
use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapConnections\DeleteMapConnectionAction;
use App\Actions\MapConnections\UpdateMapConnectionAction;
use App\Data\MapConnectionData;
use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Events\MapConnections\MapConnectionsRemovedEvent;
use App\Events\MapConnections\MapConnectionsUpsertedEvent;
use App\Events\Maps\MapResyncEvent;
use App\Models\Map;
use App\Models\MapConnection;
use Illuminate\Support\Facades\Event;

it('broadcasts the created connection as an entity payload', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 30022001);
    $to = placeMapSolarsystem($map, 30022002);

    Event::fake([MapConnectionsUpsertedEvent::class]);

    $connection = app(CreateMapConnectionAction::class)->handle([
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
    ]);

    Event::assertDispatched(MapConnectionsUpsertedEvent::class, function (MapConnectionsUpsertedEvent $event) use ($map, $connection, $from, $to): bool {
        $payload = $event->broadcastWith();
        $entity = $payload['map_connections'][0];

        return $event->map_id === $map->id
            && count($payload['map_connections']) === 1
            && $entity['id'] === $connection->id
            && $entity['from_map_solarsystem_id'] === $from->id
            && $entity['to_map_solarsystem_id'] === $to->id
            && count($entity['signatures']) === 0
            && array_key_exists('mass_status', $entity)
            && array_key_exists('lifetime_status', $entity);
    });
});

it('broadcasts the updated connection as an entity payload', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 30022003);
    $to = placeMapSolarsystem($map, 30022004);
    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        'mass_status' => MassStatus::Fresh,
    ]);

    Event::fake([MapConnectionsUpsertedEvent::class]);

    app(UpdateMapConnectionAction::class)->handle($connection, MapConnectionData::from(['mass_status' => MassStatus::Critical]));

    Event::assertDispatched(MapConnectionsUpsertedEvent::class, function (MapConnectionsUpsertedEvent $event) use ($map, $connection): bool {
        $entity = $event->broadcastWith()['map_connections'][0];

        return $event->map_id === $map->id
            && $entity['id'] === $connection->id
            && $entity['mass_status'] === MassStatus::Critical;
    });
});

it('broadcasts the deleted connection with the cascade-removed system ids', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 30022005);
    $to = placeMapSolarsystem($map, 30022006);
    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
    ]);

    Event::fake([MapConnectionsRemovedEvent::class]);

    app(DeleteMapConnectionAction::class)->handle($connection, remove_map_solarsystem: true);

    Event::assertDispatched(MapConnectionsRemovedEvent::class, function (MapConnectionsRemovedEvent $event) use ($map, $connection, $from, $to): bool {
        $payload = $event->broadcastWith();

        return $event->map_id === $map->id
            && $payload['map_connection_ids'] === [$connection->id]
            && collect($payload['map_solarsystem_ids'])->sort()->values()->all() === collect([$from->id, $to->id])->sort()->values()->all();
    });
});

it('broadcasts the deleted connection without system ids when nothing cascades', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 30022007);
    $to = placeMapSolarsystem($map, 30022008);
    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
    ]);

    Event::fake([MapConnectionsRemovedEvent::class]);

    app(DeleteMapConnectionAction::class)->handle($connection);

    Event::assertDispatched(MapConnectionsRemovedEvent::class, function (MapConnectionsRemovedEvent $event) use ($connection): bool {
        $payload = $event->broadcastWith();

        return $payload['map_connection_ids'] === [$connection->id]
            && $payload['map_solarsystem_ids'] === [];
    });
});

it('broadcasts a small stale cleanup as removed ids', function () {
    $map = Map::factory()->create();
    $home = placeMapSolarsystem($map, 30022100);
    $middle = placeMapSolarsystem($map, 30022101);
    $leaf = placeMapSolarsystem($map, 30022102);
    $map->update(['home_solarsystem_id' => 30022100]);

    MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $home->id,
        'to_map_solarsystem_id' => $middle->id,
        'lifetime' => LifetimeStatus::Healthy,
        'lifetime_updated_at' => null,
    ]);
    $stale = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $middle->id,
        'to_map_solarsystem_id' => $leaf->id,
        'lifetime' => LifetimeStatus::Critical,
        'lifetime_updated_at' => now()->subHours(2),
    ]);

    Event::fake([MapConnectionsRemovedEvent::class, MapResyncEvent::class]);

    app(CleanStaleMapConnectionsAction::class)->handle($map);

    Event::assertNotDispatched(MapResyncEvent::class);
    Event::assertDispatched(MapConnectionsRemovedEvent::class, function (MapConnectionsRemovedEvent $event) use ($map, $stale, $leaf): bool {
        $payload = $event->broadcastWith();

        return $event->map_id === $map->id
            && $payload['map_connection_ids'] === [$stale->id]
            && $payload['map_solarsystem_ids'] === [$leaf->id];
    });
});

it('broadcasts a resync ping when the stale cleanup exceeds the id limit', function () {
    $map = Map::factory()->create();
    $previous = placeMapSolarsystem($map, 30022200);
    $map->update(['home_solarsystem_id' => 30022200]);

    foreach (range(1, 21) as $index) {
        $next = placeMapSolarsystem($map, 30022200 + $index);
        MapConnection::factory()->create([
            'map_id' => $map->id,
            'from_map_solarsystem_id' => $previous->id,
            'to_map_solarsystem_id' => $next->id,
            'lifetime' => LifetimeStatus::Critical,
            'lifetime_updated_at' => now()->subHours(2),
        ]);
        $previous = $next;
    }

    Event::fake([MapConnectionsRemovedEvent::class, MapResyncEvent::class]);

    app(CleanStaleMapConnectionsAction::class)->handle($map);

    Event::assertNotDispatched(MapConnectionsRemovedEvent::class);
    Event::assertDispatched(MapResyncEvent::class, fn (MapResyncEvent $event): bool => $event->map_id === $map->id);
});
