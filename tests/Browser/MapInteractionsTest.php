<?php

declare(strict_types=1);

use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\MapSolarsystemDetails;

/**
 * Real EVE systems that exist in the bundled static data (resources/static/solarsystems.json),
 * which the client-side search reads from. They must also exist in the DB for the FK.
 */
const JITA = 30000142;
const AMARR = 30002187;

it('renders a placed system on the canvas with the onboarding wizard dismissed', function () {
    $map = Map::factory()->create();
    $this->actAsMapOwner($map);
    $this->placeSystem($map, JITA, 300, 300);

    visit(route('maps.show', $map))
        ->assertDontSee('Welcome to the Map')
        ->assertSeeIn('.bg-grid', 'Jita');
});

it('selects a system by clicking it', function () {
    $map = Map::factory()->create();
    $this->actAsMapOwner($map);
    $this->placeSystem($map, JITA, 300, 300);

    visit(route('maps.show', $map))
        ->click('[data-solarsystem-id="30000142"]')
        ->assertQueryStringHas('solarsystem_id', '30000142')
        ->assertSeeIn('[data-is-active="true"]', 'Jita');
});

it('removes a system via its context menu and keeps the persistent details', function () {
    $map = Map::factory()->create();
    $this->actAsMapOwner($map);
    $this->placeSystem($map, JITA, 300, 300);

    expect(MapSolarsystem::where('map_id', $map->id)->count())->toBe(1);

    visit(route('maps.show', $map))
        ->rightClick('[data-solarsystem-id="30000142"]')
        ->click('Remove')
        ->assertDontSee('Jita');

    expect(MapSolarsystem::where('map_id', $map->id)->count())->toBe(0)
        ->and(MapSolarsystemDetails::where('map_id', $map->id)->where('solarsystem_id', JITA)->exists())->toBeTrue();
});

it('adds a system to the map via the status bar search', function () {
    $map = Map::factory()->create();
    $this->actAsMapOwner($map);
    makeSolarsystem(JITA);

    expect(MapSolarsystem::where('map_id', $map->id)->count())->toBe(0);

    visit(route('maps.show', $map))
        ->type('[data-slot="command-input"]', 'Jita')
        ->click('Jita')
        ->assertSeeIn('.bg-grid', 'Jita');

    expect(MapSolarsystem::where('map_id', $map->id)->where('solarsystem_id', JITA)->exists())->toBeTrue();
});

it('draws a connection between two systems from the connection handle', function () {
    $map = Map::factory()->create();
    $this->actAsMapOwner($map);
    $this->placeSystem($map, JITA, 200, 200);
    $this->placeSystem($map, AMARR, 600, 200);

    expect(MapConnection::where('map_id', $map->id)->count())->toBe(0);

    $this->connectSystems(visit(route('maps.show', $map)), JITA, AMARR);

    // The connection is persisted via an async POST; wait briefly for it to land.
    $deadline = microtime(true) + 5;
    while (MapConnection::where('map_id', $map->id)->count() === 0 && microtime(true) < $deadline) {
        usleep(100_000);
    }

    expect(MapConnection::where('map_id', $map->id)->count())->toBe(1);
});

it('prefills the alias editor with an alias assigned after the page loaded', function () {
    $map = Map::factory()->create();
    $this->actAsMapOwner($map);
    $system = $this->placeSystem($map, JITA, 300, 300);

    $page = visit(route('maps.show', $map))
        ->assertSeeIn('.bg-grid', 'Jita');

    // The tracker (or another user) assigns the alias while the map is open.
    $system->update(['alias' => 'TRACKED']);

    // Selecting the system refreshes the map prop; the card renders the alias.
    $page->click('[data-solarsystem-id="30000142"]')
        ->assertSeeIn('.bg-grid', 'TRACKED');

    // The editor must open prefilled with the alias, not the mount-time snapshot.
    $page->script('document.querySelector(\'[data-solarsystem-id="30000142"]\').dispatchEvent(new MouseEvent("dblclick", { bubbles: true }))');

    $page->assertValue('input[placeholder="Alias"]', 'TRACKED');
});

it('round-trips alias and occupier through the alias editor', function () {
    $map = Map::factory()->create();
    $this->actAsMapOwner($map);
    $system = $this->placeSystem($map, JITA, 300, 300);

    $page = visit(route('maps.show', $map))
        ->assertSeeIn('.bg-grid', 'Jita');

    $page->script('document.querySelector(\'[data-solarsystem-id="30000142"]\').dispatchEvent(new MouseEvent("dblclick", { bubbles: true }))');

    $page->fill('input[placeholder="Alias"]', 'HOME')
        ->fill('input[placeholder="Occupier Alias"]', 'Lazerhawks')
        ->click('Save');

    // The update is persisted via an async PUT; wait briefly for it to land.
    $deadline = microtime(true) + 5;
    while ($system->fresh()->alias === null && microtime(true) < $deadline) {
        usleep(100_000);
    }

    $system = $system->fresh()->loadMissing('details');
    expect($system->alias)->toBe('HOME')
        ->and($system->details->occupier_alias)->toBe('Lazerhawks');

    // Wait for the reload to render (the success handler also closes the
    // popover — reopening before that would race it), then reopen.
    $page->assertSeeIn('.bg-grid', 'HOME');
    $page->script('document.querySelector(\'[data-solarsystem-id="30000142"]\').dispatchEvent(new MouseEvent("dblclick", { bubbles: true }))');

    $page->assertValue('input[placeholder="Alias"]', 'HOME')
        ->assertValue('input[placeholder="Occupier Alias"]', 'Lazerhawks');
});
