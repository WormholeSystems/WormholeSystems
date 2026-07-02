<?php

declare(strict_types=1);

use App\Actions\MapSelection\DeleteMapSelectionAction;
use App\Actions\MapSelection\UpdateMapSelectionAction;
use App\Events\Maps\MapResyncEvent;
use App\Events\MapSolarsystems\MapSolarsystemsRemovedEvent;
use App\Events\MapSolarsystems\MapSolarsystemsUpsertedEvent;
use App\Models\Map;
use App\Models\MapConnection;
use Illuminate\Support\Facades\Event;

it('broadcasts a small selection move as an entity payload', function () {
    $map = Map::factory()->create();
    $first = placeMapSolarsystem($map, 30021001);
    $second = placeMapSolarsystem($map, 30021002);

    Event::fake([MapSolarsystemsUpsertedEvent::class, MapResyncEvent::class]);

    app(UpdateMapSelectionAction::class)->handle([
        'map_solarsystems' => [
            ['id' => $first->id, 'position_x' => 500, 'position_y' => 600],
            ['id' => $second->id, 'position_x' => 700, 'position_y' => 800],
        ],
    ]);

    Event::assertNotDispatched(MapResyncEvent::class);
    Event::assertDispatched(MapSolarsystemsUpsertedEvent::class, function (MapSolarsystemsUpsertedEvent $event) use ($map, $first): bool {
        $payload = $event->broadcastWith();
        $moved = collect($payload['map_solarsystems'])->firstWhere('id', $first->id);

        return $event->map_id === $map->id
            && count($payload['map_solarsystems']) === 2
            && $moved['position'] === ['x' => 500, 'y' => 600];
    });
});

it('broadcasts a resync ping when the selection move exceeds the payload limit', function () {
    $map = Map::factory()->create();

    $systems = collect(range(1, 11))->map(fn (int $index) => placeMapSolarsystem($map, 30021100 + $index));

    Event::fake([MapSolarsystemsUpsertedEvent::class, MapResyncEvent::class]);

    app(UpdateMapSelectionAction::class)->handle([
        'map_solarsystems' => $systems
            ->map(fn ($system) => ['id' => $system->id, 'position_x' => 10, 'position_y' => 20])
            ->all(),
    ]);

    Event::assertNotDispatched(MapSolarsystemsUpsertedEvent::class);
    Event::assertDispatched(MapResyncEvent::class, fn (MapResyncEvent $event): bool => $event->map_id === $map->id
        && $event->broadcastWith() === []);
});

it('broadcasts removed ids for a deleted selection', function () {
    $map = Map::factory()->create();
    $first = placeMapSolarsystem($map, 30021201);
    $second = placeMapSolarsystem($map, 30021202);
    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $first->id,
        'to_map_solarsystem_id' => $second->id,
    ]);

    Event::fake([MapSolarsystemsRemovedEvent::class]);

    app(DeleteMapSelectionAction::class)->handle([$first->id, $second->id]);

    Event::assertDispatched(MapSolarsystemsRemovedEvent::class, function (MapSolarsystemsRemovedEvent $event) use ($map, $first, $second, $connection): bool {
        $payload = $event->broadcastWith();

        return $event->map_id === $map->id
            && collect($payload['map_solarsystem_ids'])->sort()->values()->all() === collect([$first->id, $second->id])->sort()->values()->all()
            && $payload['map_connection_ids'] === [$connection->id];
    });
});
