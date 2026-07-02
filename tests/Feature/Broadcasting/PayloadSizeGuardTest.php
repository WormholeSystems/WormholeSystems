<?php

declare(strict_types=1);

use App\Enums\MapSolarsystemStatus;
use App\Events\MapSolarsystems\MapSolarsystemsUpsertedEvent;
use App\Models\Map;
use App\Models\MapSolarsystem;

it('keeps a maxed-out system upsert payload under the reverb message size cap', function () {
    $map = Map::factory()->create();

    $system_ids = collect(range(1, 10))->map(function (int $index) use ($map): int {
        $system = placeMapSolarsystem($map, 30025000 + $index);
        $system->update(['alias' => str_repeat('A', 100), 'pinned' => true]);
        $system->details->update([
            'occupier_alias' => str_repeat('B', 100),
            'status' => MapSolarsystemStatus::Hostile,
        ]);

        return $system->id;
    });

    $systems = MapSolarsystem::query()
        ->whereIn('id', $system_ids)
        ->with('details')
        ->withCount('signatures', 'wormholeSignatures', 'mapConnections', 'uncategorizedSignatures')
        ->get();

    $event = new MapSolarsystemsUpsertedEvent($map->id, $systems);

    expect(mb_strlen(json_encode($event->broadcastWith())))->toBeLessThan(10_000);
});
