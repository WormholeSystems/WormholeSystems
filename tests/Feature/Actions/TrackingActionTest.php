<?php

declare(strict_types=1);

use App\Actions\MapSolarsystem\UpdateMapSolarsystemAction;
use App\Actions\Tracking\StoreTrackingAction;
use App\Data\TrackingData;
use App\Events\MapSolarsystems\MapSolarsystemsUpsertedEvent;
use App\Models\Map;
use App\Models\MapConnection;
use Illuminate\Support\Facades\Event;

it('adds the target system and connects it when tracking a jump', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012001);
    $targetId = makeSolarsystem(30012002);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
    ]));

    expect(MapConnection::where('map_id', $map->id)->count())->toBe(1)
        ->and($map->mapSolarsystems()->where('solarsystem_id', $targetId)->whereNotNull('position_x')->exists())->toBeTrue();
});

it('does not duplicate a connection that already exists', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012003);
    $targetId = makeSolarsystem(30012004);

    $data = TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
    ]);

    app(StoreTrackingAction::class)->handle($data);
    app(StoreTrackingAction::class)->handle($data);

    expect(MapConnection::where('map_id', $map->id)->count())->toBe(1);
});

it('assigns the alias to a tracked system and broadcasts it', function () {
    Event::fake([MapSolarsystemsUpsertedEvent::class]);

    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012005);
    $targetId = makeSolarsystem(30012006);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
        'alias' => 'C2a',
    ]));

    $target = $map->mapSolarsystems()->where('solarsystem_id', $targetId)->firstOrFail();
    expect($target->alias)->toBe('C2a');

    Event::assertDispatched(MapSolarsystemsUpsertedEvent::class, function (MapSolarsystemsUpsertedEvent $event) use ($target): bool {
        return collect($event->broadcastWith()['map_solarsystems'])
            ->contains(fn (array $system): bool => $system['id'] === $target->id && $system['alias'] === 'C2a');
    });
});

it('assigns the alias when the tracked system is already on the map', function () {
    Event::fake([MapSolarsystemsUpsertedEvent::class]);

    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012007);
    $target = placeMapSolarsystem($map, 30012008, 300, 300);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $target->solarsystem_id,
        'alias' => 'STATIC',
    ]));

    expect($target->fresh()->alias)->toBe('STATIC');

    Event::assertDispatched(MapSolarsystemsUpsertedEvent::class, function (MapSolarsystemsUpsertedEvent $event) use ($target): bool {
        return collect($event->broadcastWith()['map_solarsystems'])
            ->contains(fn (array $system): bool => $system['id'] === $target->id && $system['alias'] === 'STATIC');
    });
});

it('updates the alias and occupier of a tracked system afterwards', function () {
    Event::fake([MapSolarsystemsUpsertedEvent::class]);

    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012011);
    $targetId = makeSolarsystem(30012012);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
        'alias' => 'C3',
    ]));

    $target = $map->mapSolarsystems()->where('solarsystem_id', $targetId)->firstOrFail();

    app(UpdateMapSolarsystemAction::class)->handle($target, [
        'alias' => 'STAGING',
        'occupier_alias' => 'Lazerhawks',
    ]);

    $target = $target->fresh()->loadMissing('details');
    expect($target->alias)->toBe('STAGING')
        ->and($target->details->occupier_alias)->toBe('Lazerhawks');

    Event::assertDispatched(MapSolarsystemsUpsertedEvent::class, function (MapSolarsystemsUpsertedEvent $event) use ($target): bool {
        return collect($event->broadcastWith()['map_solarsystems'])
            ->contains(fn (array $system): bool => $system['id'] === $target->id
                && $system['alias'] === 'STAGING'
                && $system['occupier_alias'] === 'Lazerhawks');
    });
});
