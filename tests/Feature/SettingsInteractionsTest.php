<?php

declare(strict_types=1);

use App\Models\Character;
use App\Models\EsiToken;
use App\Models\User;

it('does not remove a character belonging to another user', function () {
    $user = User::factory()->has(Character::factory()->count(2))->create();
    $otherUser = User::factory()->has(Character::factory())->create();
    $character = $otherUser->characters()->sole();

    $this->actingAs($user)
        ->delete(route('user-characters.delete', $character))
        ->assertRedirect(route('home'));

    expect($character->refresh()->user_id)->toBe($otherUser->id)
        ->and($user->characters()->count())->toBe(2);
});

it('selects a remaining character when removing the preferred character', function () {
    $user = User::factory()->has(Character::factory()->count(2))->create();
    $removedCharacter = $user->characters()->firstOrFail();
    $remainingCharacter = $user->characters()->whereKeyNot($removedCharacter->id)->sole();
    $user->update(['preferred_character_id' => $removedCharacter->id]);

    $this->actingAs($user)
        ->delete(route('user-characters.delete', $removedCharacter))
        ->assertRedirect()
        ->assertSessionHas('active_character_id', $remainingCharacter->id);

    expect($removedCharacter->refresh()->user_id)->toBeNull()
        ->and($user->refresh()->preferred_character_id)->toBe($remainingCharacter->id);
});

it('only allows an owned character to become preferred', function () {
    $user = User::factory()->has(Character::factory())->create();
    $ownedCharacter = $user->characters()->sole();
    $otherCharacter = Character::factory()->create();

    $this->actingAs($user)
        ->post(route('preferred-character.store', $otherCharacter))
        ->assertRedirect();

    expect($user->refresh()->preferred_character_id)->toBeNull();

    $this->actingAs($user)
        ->post(route('preferred-character.store', $ownedCharacter))
        ->assertRedirect()
        ->assertSessionHas('active_character_id', $ownedCharacter->id);

    expect($user->refresh()->preferred_character_id)->toBe($ownedCharacter->id);
});

it('only removes scopes from an owned character', function () {
    $user = User::factory()->has(Character::factory())->create();
    $ownedCharacter = $user->characters()->sole();
    $otherCharacter = Character::factory()->create();
    $ownedToken = createEsiToken($ownedCharacter);
    $otherToken = createEsiToken($otherCharacter);

    $this->actingAs($user)
        ->delete(route('scopes.destroy', $otherCharacter))
        ->assertForbidden();

    expect($otherToken->fresh())->not->toBeNull();

    $this->actingAs($user)
        ->delete(route('scopes.destroy', $ownedCharacter))
        ->assertRedirect();

    expect($ownedToken->fresh())->toBeNull()
        ->and($otherToken->fresh())->not->toBeNull();
});

it('only deletes API tokens owned by the current user', function () {
    $user = User::factory()->create();
    $otherUser = User::factory()->create();
    $ownedToken = $user->createToken('Owned')->accessToken;
    $otherToken = $otherUser->createToken('Other')->accessToken;

    $this->actingAs($user)
        ->delete(route('tokens.destroy', $otherToken))
        ->assertForbidden();

    expect($otherToken->fresh())->not->toBeNull();

    $this->actingAs($user)
        ->delete(route('tokens.destroy', $ownedToken))
        ->assertRedirect();

    expect($ownedToken->fresh())->toBeNull()
        ->and($otherToken->fresh())->not->toBeNull();
});

function createEsiToken(Character $character): EsiToken
{
    return EsiToken::query()->forceCreate([
        'character_id' => $character->id,
        'character_owner_hash' => fake()->sha256(),
        'access_token' => fake()->sha256(),
        'refresh_token' => fake()->sha256(),
        'expires_at' => now()->addHour(),
    ]);
}
