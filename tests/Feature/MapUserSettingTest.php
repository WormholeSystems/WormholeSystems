<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Features\MapSettingsFeature;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapUserSetting;
use App\Models\User;

use function Pest\Laravel\actingAs;

// --- Guest can view a public map without error ---

it('allows a guest to view a public map without error', function () {
    $map = Map::factory()->create(['is_public' => true]);
    User::factory()->ownsMap($map)->create();

    $this->withoutExceptionHandling()
        ->get(route('maps.show', $map))
        ->assertSuccessful();
});

// --- Guest settings come from session defaults ---

it('provides default settings for a guest viewing a public map', function () {
    $map = Map::factory()->create(['is_public' => true]);
    User::factory()->ownsMap($map)->create();

    $this->withoutExceptionHandling()
        ->get(route('maps.show', $map))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page
            ->has('map_user_settings', fn ($settings) => $settings
                ->where('killmail_filter', 'all')
                ->where('id', null)
                ->where('user_id', null)
                ->etc()
            )
        );
});

// --- Guest can update settings via session ---

it('allows a guest to update settings on a public map', function () {
    $map = Map::factory()->create(['is_public' => true]);

    $this->put(route('maps.user-settings.update', $map), [
        'killmail_filter' => 'jspace',
    ])->assertRedirect();

    // Verify session was updated
    $sessionKey = MapSettingsFeature::sessionKey($map->id);
    expect(session($sessionKey)['killmail_filter'])->toBe('jspace');
});

it('persists guest settings across page reloads via session', function () {
    $map = Map::factory()->create(['is_public' => true]);
    User::factory()->ownsMap($map)->create();

    // First set a preference
    $this->put(route('maps.user-settings.update', $map), [
        'killmail_filter' => 'kspace',
    ])->assertRedirect();

    // Then view the map — settings should come from session
    $this->withoutExceptionHandling()
        ->get(route('maps.show', $map))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page
            ->has('map_user_settings', fn ($settings) => $settings
                ->where('killmail_filter', 'kspace')
                ->etc()
            )
        );
});

// --- Guest does not create DB records ---

it('does not create a database record when a guest updates settings', function () {
    $map = Map::factory()->create(['is_public' => true]);

    $this->put(route('maps.user-settings.update', $map), [
        'killmail_filter' => 'jspace',
    ]);

    expect(MapUserSetting::count())->toBe(0);
});

// --- Authenticated user settings persist to DB ---

it('persists settings to the database for an authenticated user', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))->create();

    actingAs($user);

    $this->put(route('maps.user-settings.update', $map), [
        'killmail_filter' => 'jspace',
    ])->assertRedirect();

    $settings = MapUserSetting::query()
        ->where('user_id', $user->id)
        ->where('map_id', $map->id)
        ->first();

    expect($settings)->not->toBeNull()
        ->and($settings->killmail_filter->value)->toBe('jspace');
});

it('also updates the session for an authenticated user', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))->create();

    actingAs($user);

    $this->put(route('maps.user-settings.update', $map), [
        'killmail_filter' => 'kspace',
    ])->assertRedirect();

    $sessionKey = MapSettingsFeature::sessionKey($map->id);
    expect(session($sessionKey)['killmail_filter'])->toBe('kspace');
});

// --- Auth user loads settings from DB on first visit ---

it('loads settings from the database on first visit for an authenticated user', function () {
    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();

    // Set preferred character so active_character resolves properly
    $character = $user->characters->first();
    $user->update(['preferred_character_id' => $character->id]);

    actingAs($user);

    $response = $this->get(route('maps.show', $map));
    $response->assertSuccessful();

    // A DB record should now exist
    expect(MapUserSetting::query()
        ->where('user_id', $user->id)
        ->where('map_id', $map->id)
        ->exists()
    )->toBeTrue();
});

// --- Authorization: user without access cannot update ---

it('prevents an authenticated user without access from updating settings', function () {
    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory())->create();

    actingAs($user);

    $this->put(route('maps.user-settings.update', $map), [
        'killmail_filter' => 'jspace',
    ])->assertForbidden();
});

// --- Guest cannot update private map settings ---

it('prevents a guest from updating settings on a private map', function () {
    $map = Map::factory()->create(['is_public' => false]);

    $this->put(route('maps.user-settings.update', $map), [
        'killmail_filter' => 'jspace',
    ])->assertForbidden();
});
