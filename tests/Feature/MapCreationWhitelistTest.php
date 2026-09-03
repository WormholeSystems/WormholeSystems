<?php

declare(strict_types=1);

use App\Models\Alliance;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\Map;
use App\Models\User;

use function Pest\Laravel\actingAs;

it('lets any authenticated user create a map when no whitelist is configured', function () {
    config()->set('access.map_creator_affiliation_ids', []);

    $user = User::factory()->has(Character::factory())->create();

    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    actingAs($user)
        ->post(route('maps.store'), ['name' => 'Home chain'])
        ->assertRedirect();

    expect(Map::query()->where('name', 'Home chain')->exists())->toBeTrue();
});

it('lets a whitelisted character create a map', function () {
    $user = User::factory()->has(Character::factory())->create();
    $character = $user->characters->first();

    config()->set('access.map_creator_affiliation_ids', [$character->id]);

    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    actingAs($user)
        ->post(route('maps.store'), ['name' => 'Home chain'])
        ->assertRedirect();

    expect(Map::query()->where('name', 'Home chain')->exists())->toBeTrue();
});

it('lets a member of a whitelisted corporation create a map', function () {
    $corporation = Corporation::factory()->create();
    $user = User::factory()->has(Character::factory()->for($corporation))->create();

    config()->set('access.map_creator_affiliation_ids', [$corporation->id]);

    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    actingAs($user)
        ->post(route('maps.store'), ['name' => 'Home chain'])
        ->assertRedirect();

    expect(Map::query()->where('name', 'Home chain')->exists())->toBeTrue();
});

it('lets a member of a whitelisted alliance create a map', function () {
    $alliance = Alliance::factory()->create();
    $corporation = Corporation::factory()->for($alliance)->create();
    $user = User::factory()->has(Character::factory()->for($corporation)->for($alliance))->create();

    config()->set('access.map_creator_affiliation_ids', [$alliance->id]);

    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    actingAs($user)
        ->post(route('maps.store'), ['name' => 'Home chain'])
        ->assertRedirect();

    expect(Map::query()->where('name', 'Home chain')->exists())->toBeTrue();
});

it('stops a user outside the whitelist from creating a map', function () {
    $user = User::factory()->has(Character::factory())->create();

    // An affiliation the user has nothing to do with.
    config()->set('access.map_creator_affiliation_ids', [99_999_999]);

    actingAs($user->fresh())
        ->post(route('maps.store'), ['name' => 'Home chain'])
        ->assertForbidden();

    expect(Map::query()->where('name', 'Home chain')->exists())->toBeFalse();
});

it('does not affect who may view or edit an existing map', function () {
    // The whitelist governs creation only: a user who cannot create a map can
    // still work on one they have been given access to.
    config()->set('access.map_creator_affiliation_ids', [99_999_999]);

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();
    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    actingAs($user)
        ->get(route('maps.show', $map))
        ->assertSuccessful();
});

it('checks the active character, not every character on the account', function () {
    // An account can hold a director and an alt in an NPC corporation. Playing
    // the alt should not inherit the director's right to create maps.
    $allowed = Corporation::factory()->create();
    $user = User::factory()->has(Character::factory()->for($allowed))->create();
    $director = $user->characters->first();
    $alt = Character::factory()->for(Corporation::factory()->create())->create(['user_id' => $user->id]);

    config()->set('access.map_creator_affiliation_ids', [$allowed->id]);

    $user->update(['preferred_character_id' => $alt->id]);
    actingAs($user->fresh())
        ->post(route('maps.store'), ['name' => 'From the alt'])
        ->assertForbidden();

    $user->update(['preferred_character_id' => $director->id]);
    actingAs($user->fresh())
        ->post(route('maps.store'), ['name' => 'From the director'])
        ->assertRedirect();

    expect(Map::query()->where('name', 'From the alt')->exists())->toBeFalse()
        ->and(Map::query()->where('name', 'From the director')->exists())->toBeTrue();
});

it('tells the maps page whether the user may create one', function () {
    $user = User::factory()->has(Character::factory())->create();
    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    config()->set('access.map_creator_affiliation_ids', []);

    actingAs($user)
        ->get(route('home'))
        ->assertInertia(fn ($page) => $page->where('can_create_map', true)->etc());

    config()->set('access.map_creator_affiliation_ids', [99_999_999]);

    actingAs($user)
        ->get(route('home'))
        ->assertInertia(fn ($page) => $page->where('can_create_map', false)->etc());
});
