<?php

declare(strict_types=1);

use App\Actions\Map\UpdateMapAction;
use App\Events\Maps\MapMetadataUpdatedEvent;
use App\Models\Map;
use Illuminate\Support\Facades\Event;

it('broadcasts the map metadata without the topology when the map is updated', function () {
    $map = Map::factory()->create(['name' => 'Old Name']);
    placeMapSolarsystem($map, 30024001);

    Event::fake([MapMetadataUpdatedEvent::class]);

    app(UpdateMapAction::class)->handle($map, ['name' => 'New Name']);

    Event::assertDispatched(MapMetadataUpdatedEvent::class, function (MapMetadataUpdatedEvent $event) use ($map): bool {
        $payload = $event->broadcastWith();

        return $event->map_id === $map->id
            && $payload['map']['id'] === $map->id
            && $payload['map']['name'] === 'New Name'
            && array_key_exists('bookmark_format_wormhole', $payload['map'])
            && array_key_exists('layout', $payload['map'])
            && ! array_key_exists('map_solarsystems', $payload['map'])
            && ! array_key_exists('map_connections', $payload['map'])
            && ! array_key_exists('owner', $payload['map']);
    });
});
