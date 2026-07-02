<?php

declare(strict_types=1);

use App\Actions\MapSolarsystem\DeleteMapSolarsystemAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Actions\MapSolarsystem\UpdateMapSolarsystemAction;
use App\Events\MapSolarsystems\MapSolarsystemsRemovedEvent;
use App\Events\MapSolarsystems\MapSolarsystemsUpsertedEvent;
use App\Models\Map;
use App\Models\MapConnection;
use Illuminate\Support\Facades\Event;

it('broadcasts the stored system as an entity payload', function () {
    $map = Map::factory()->create();
    makeSolarsystem(30020001);

    Event::fake([MapSolarsystemsUpsertedEvent::class]);

    $system = app(StoreMapSolarsystemAction::class)->handle($map, [
        'solarsystem_id' => 30020001,
        'position_x' => 100,
        'position_y' => 200,
    ]);

    Event::assertDispatched(MapSolarsystemsUpsertedEvent::class, function (MapSolarsystemsUpsertedEvent $event) use ($map, $system): bool {
        $payload = $event->broadcastWith();
        $entity = $payload['map_solarsystems'][0];

        return $event->map_id === $map->id
            && count($payload['map_solarsystems']) === 1
            && $entity['id'] === $system->id
            && $entity['solarsystem_id'] === 30020001
            && $entity['position'] === ['x' => 100, 'y' => 200]
            && $entity['signatures_count'] === 0
            && array_key_exists('map_connections_count', $entity)
            && array_key_exists('status', $entity);
    });
});

it('broadcasts the updated system as an entity payload', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30020002);

    Event::fake([MapSolarsystemsUpsertedEvent::class]);

    app(UpdateMapSolarsystemAction::class)->handle($system, [
        'alias' => 'Staging',
        'occupier_alias' => 'Some Corp',
    ]);

    Event::assertDispatched(MapSolarsystemsUpsertedEvent::class, function (MapSolarsystemsUpsertedEvent $event) use ($map, $system): bool {
        $entity = $event->broadcastWith()['map_solarsystems'][0];

        return $event->map_id === $map->id
            && $entity['id'] === $system->id
            && $entity['alias'] === 'Staging'
            && $entity['occupier_alias'] === 'Some Corp';
    });
});

it('broadcasts removed system and connection ids captured before the delete cascades', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30020003);
    $other = placeMapSolarsystem($map, 30020004);
    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $system->id,
        'to_map_solarsystem_id' => $other->id,
    ]);

    Event::fake([MapSolarsystemsRemovedEvent::class]);

    app(DeleteMapSolarsystemAction::class)->handle($system);

    Event::assertDispatched(MapSolarsystemsRemovedEvent::class, function (MapSolarsystemsRemovedEvent $event) use ($map, $system, $connection): bool {
        $payload = $event->broadcastWith();

        return $event->map_id === $map->id
            && $payload['map_solarsystem_ids'] === [$system->id]
            && $payload['map_connection_ids'] === [$connection->id];
    });
});
