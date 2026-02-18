<?php

declare(strict_types=1);

use App\Models\Map;
use App\Models\MapUserSetting;
use App\Models\User;

use function Pest\Laravel\actingAs;

beforeEach(function () {
    $this->map = Map::factory()->create();
    $this->user = User::factory()->ownsMap($this->map)->create();

    // Set preferred character so active_character resolves properly
    $character = $this->user->characters->first();
    $this->user->update(['preferred_character_id' => $character->id]);

    actingAs($this->user);

    // Ensure map user settings exist
    $this->settings = MapUserSetting::query()->firstOrCreate([
        'user_id' => $this->user->id,
        'map_id' => $this->map->id,
    ]);
});

it('accepts valid removable card IDs in hidden_cards', function () {
    $this->putJson(route('map-user-settings.update', $this->settings), [
        'hidden_cards' => ['audits', 'ship-history', 'characters', 'killmails', 'autopilot', 'eve-scout'],
    ])->assertRedirect();

    $this->settings->refresh();
    expect($this->settings->hidden_cards)->toBe(['audits', 'ship-history', 'characters', 'killmails', 'autopilot', 'eve-scout']);
});

it('rejects invalid card IDs in hidden_cards', function () {
    $this->putJson(route('map-user-settings.update', $this->settings), [
        'hidden_cards' => ['map'],
    ])->assertUnprocessable();
});

it('rejects core card IDs in hidden_cards', function (string $coreCard) {
    $this->putJson(route('map-user-settings.update', $this->settings), [
        'hidden_cards' => [$coreCard],
    ])->assertUnprocessable();
})->with(['map', 'system-info', 'solarsystem', 'signatures']);

it('persists hidden_cards correctly via PUT', function () {
    $this->putJson(route('map-user-settings.update', $this->settings), [
        'hidden_cards' => ['killmails', 'audits'],
    ])->assertRedirect();

    $this->settings->refresh();
    expect($this->settings->hidden_cards)->toBe(['killmails', 'audits']);
});

it('allows nullable hidden_cards', function () {
    $this->settings->update(['hidden_cards' => ['killmails']]);

    $this->putJson(route('map-user-settings.update', $this->settings), [
        'hidden_cards' => null,
    ])->assertRedirect();

    $this->settings->refresh();
    expect($this->settings->hidden_cards)->toBeNull();
});

it('does not load killmails feature data when killmails card is hidden', function () {
    $this->settings->update(['hidden_cards' => ['killmails']]);

    $response = $this->get(route('maps.show', $this->map));

    $response->assertSuccessful();
    $response->assertInertia(fn ($page) => $page
        ->missing('map_killmails')
    );
});

it('does not load ship_history when ship-history card is hidden', function () {
    $this->settings->update(['hidden_cards' => ['ship-history']]);

    $response = $this->get(route('maps.show', $this->map));

    $response->assertSuccessful();
    $response->assertInertia(fn ($page) => $page
        ->missing('ship_history')
    );
});

it('does not load map_navigation when autopilot card is hidden', function () {
    $this->settings->update(['hidden_cards' => ['autopilot']]);

    $response = $this->get(route('maps.show', $this->map));

    $response->assertSuccessful();
    $response->assertInertia(fn ($page) => $page
        ->missing('map_navigation')
    );
});

it('still loads eve_scout_connections when eve-scout card is hidden', function () {
    $this->settings->update(['hidden_cards' => ['eve-scout']]);

    $response = $this->get(route('maps.show', $this->map));

    $response->assertSuccessful();
    $response->assertInertia(fn ($page) => $page
        ->has('eve_scout_connections')
    );
});

it('still loads map_characters when characters card is hidden', function () {
    $this->settings->update(['hidden_cards' => ['characters']]);

    $response = $this->get(route('maps.show', $this->map));

    $response->assertSuccessful();
    $response->assertInertia(fn ($page) => $page
        ->has('map_characters')
    );
});

it('loads all features when no cards are hidden', function () {
    $response = $this->get(route('maps.show', $this->map));

    $response->assertSuccessful();
    // map_killmails uses Inertia::defer() so is not in initial props
    $response->assertInertia(fn ($page) => $page
        ->has('map_navigation')
        ->has('eve_scout_connections')
        ->has('map_characters')
    );
});
