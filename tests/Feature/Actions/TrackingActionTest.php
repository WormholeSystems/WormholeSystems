<?php

declare(strict_types=1);

use App\Actions\MapSolarsystem\UpdateMapSolarsystemAction;
use App\Actions\Tracking\StoreTrackingAction;
use App\Data\TrackingData;
use App\Enums\ShipSize;
use App\Events\MapSolarsystems\MapSolarsystemsUpsertedEvent;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\Signature;
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

it('applies an explicitly chosen ship size to the tracked connection', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012005);
    $targetId = makeSolarsystem(30012006);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
        'ship_size' => 'frigate',
    ]));

    expect(MapConnection::where('map_id', $map->id)->value('ship_size'))->toBe(ShipSize::Frigate);
});

it('falls back to the signature ship size when none is chosen explicitly', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012007);
    $targetId = makeSolarsystem(30012008);

    $signature = Signature::create([
        'map_solarsystem_id' => $origin->id,
        'signature_id' => 'ABC-123',
        'ship_size' => 'medium',
    ]);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
        'signature_id' => $signature->id,
    ]));

    expect(MapConnection::where('map_id', $map->id)->value('ship_size'))->toBe(ShipSize::Medium);
});

it('prefers the explicit ship size over the signature ship size', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012009);
    $targetId = makeSolarsystem(30012010);

    $signature = Signature::create([
        'map_solarsystem_id' => $origin->id,
        'signature_id' => 'DEF-456',
        'ship_size' => 'medium',
    ]);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
        'signature_id' => $signature->id,
        'ship_size' => 'xlarge',
    ]));

    expect(MapConnection::where('map_id', $map->id)->value('ship_size'))->toBe(ShipSize::ExtraLarge);
});

it('locks the connection ship size to the signature wormhole type', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30012011);
    $targetId = makeSolarsystem(30012012);

    $wormhole = App\Models\Wormhole::create([
        'name' => 'H296',
        'total_mass' => 3_300_000_000,
        'maximum_jump_mass' => 2_000_000_000,
        'ship_size' => '',
        'maximum_lifetime' => 86_400,
        'leads_to' => 'c5',
    ]);

    $signature = Signature::create([
        'map_solarsystem_id' => $origin->id,
        'signature_id' => 'GHI-789',
        'wormhole_id' => $wormhole->id,
    ]);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $targetId,
        'signature_id' => $signature->id,
        'ship_size' => 'frigate',
    ]));

    expect(MapConnection::where('map_id', $map->id)->value('ship_size'))->toBe(ShipSize::ExtraLarge);
});
