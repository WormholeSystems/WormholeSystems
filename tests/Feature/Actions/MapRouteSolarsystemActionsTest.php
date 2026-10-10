<?php

declare(strict_types=1);

use App\Actions\MapRouteSolarsystems\CreateMapRouteSolarsystemAction;
use App\Actions\MapRouteSolarsystems\DeleteMapRouteSolarsystemAction;
use App\Actions\MapRouteSolarsystems\UpdateMapRouteSolarsystemAction;
use App\Events\MapRouteSolarsystemsUpdatedEvent;
use App\Events\MapUserRouteSolarsystemsUpdatedEvent;
use App\Models\Map;
use App\Models\MapRouteSolarsystem;
use App\Models\User;
use Illuminate\Support\Facades\Event;

it('adds a route waypoint to a map', function () {
    $map = Map::factory()->create();

    $route = app(CreateMapRouteSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'solarsystem_id' => makeSolarsystem(30006001),
        'is_pinned' => true,
    ]);

    expect($route->map_id)->toBe($map->id)
        ->and($route->user_id)->toBeNull()
        ->and($route->is_pinned)->toBeTrue();
});

it('updates a route waypoint', function () {
    $map = Map::factory()->create();
    $route = app(CreateMapRouteSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'solarsystem_id' => makeSolarsystem(30006002),
        'is_pinned' => true,
    ]);

    app(UpdateMapRouteSolarsystemAction::class)->handle($route, ['is_pinned' => false]);

    expect($route->fresh()->is_pinned)->toBeFalse();
});

it('removes a route waypoint', function () {
    $map = Map::factory()->create();
    $route = app(CreateMapRouteSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'solarsystem_id' => makeSolarsystem(30006003),
        'is_pinned' => true,
    ]);

    app(DeleteMapRouteSolarsystemAction::class)->handle($route);

    expect(MapRouteSolarsystem::find($route->id))->toBeNull();
});

it('keeps a single shared row when the same system is added twice, updating its pin', function () {
    $map = Map::factory()->create();
    $solarsystemId = makeSolarsystem(30006004);
    $action = app(CreateMapRouteSolarsystemAction::class);

    $first = $action->handle(['map_id' => $map->id, 'solarsystem_id' => $solarsystemId, 'is_pinned' => false]);
    $second = $action->handle(['map_id' => $map->id, 'solarsystem_id' => $solarsystemId, 'is_pinned' => true]);

    expect($second->id)->toBe($first->id)
        ->and(MapRouteSolarsystem::query()->where('map_id', $map->id)->count())->toBe(1)
        ->and($first->fresh()->is_pinned)->toBeTrue();
});

it('keeps the pin when the same system is added again without one', function () {
    $map = Map::factory()->create();
    $solarsystemId = makeSolarsystem(30006005);
    $action = app(CreateMapRouteSolarsystemAction::class);

    $route = $action->handle(['map_id' => $map->id, 'solarsystem_id' => $solarsystemId, 'is_pinned' => true]);
    $action->handle(['map_id' => $map->id, 'solarsystem_id' => $solarsystemId]);

    expect($route->fresh()->is_pinned)->toBeTrue();
});

it('broadcasts shared changes on the map channel only', function () {
    $map = Map::factory()->create();
    Event::fake([MapRouteSolarsystemsUpdatedEvent::class, MapUserRouteSolarsystemsUpdatedEvent::class]);

    $route = app(CreateMapRouteSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'solarsystem_id' => makeSolarsystem(30006006),
    ]);
    app(UpdateMapRouteSolarsystemAction::class)->handle($route, ['is_pinned' => true]);
    app(DeleteMapRouteSolarsystemAction::class)->handle($route);

    Event::assertDispatchedTimes(MapRouteSolarsystemsUpdatedEvent::class, 3);
    Event::assertDispatched(MapRouteSolarsystemsUpdatedEvent::class, fn (MapRouteSolarsystemsUpdatedEvent $event): bool => $event->broadcastOn()[0]->name === sprintf('private-Map.%d', $map->id));
    Event::assertNotDispatched(MapUserRouteSolarsystemsUpdatedEvent::class);
});

it('broadcasts personal changes on the owner channel only', function () {
    $map = Map::factory()->create();
    $user = User::factory()->create();
    Event::fake([MapRouteSolarsystemsUpdatedEvent::class, MapUserRouteSolarsystemsUpdatedEvent::class]);

    $route = app(CreateMapRouteSolarsystemAction::class)->handle([
        'map_id' => $map->id,
        'user_id' => $user->id,
        'solarsystem_id' => makeSolarsystem(30006007),
    ]);
    app(UpdateMapRouteSolarsystemAction::class)->handle($route, ['is_pinned' => true]);
    app(DeleteMapRouteSolarsystemAction::class)->handle($route);

    Event::assertDispatchedTimes(MapUserRouteSolarsystemsUpdatedEvent::class, 3);
    Event::assertDispatched(
        MapUserRouteSolarsystemsUpdatedEvent::class,
        fn (MapUserRouteSolarsystemsUpdatedEvent $event): bool => $event->map_id === $map->id
            && $event->user_id === $user->id
            && $event->broadcastOn()[0]->name === sprintf('private-User.%d', $user->id),
    );
    Event::assertNotDispatched(MapRouteSolarsystemsUpdatedEvent::class);
});
