<?php

declare(strict_types=1);

use App\Models\Character;
use App\Models\Map;
use App\Models\User;

use function Pest\Laravel\actingAs;

it('creates a map from the dialog and redirects to it', function () {
    $user = User::factory()->has(Character::factory())->create();
    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    actingAs($user);

    $map_name = 'Fresh Chain';
    $response = $this->post(route('maps.store'), ['name' => $map_name]);

    $map = Map::query()->where('name', $map_name)->first();

    expect($map)->not->toBeNull();
    $response->assertRedirect(route('maps.show', $map));
});

it('requires a name to create a map', function () {
    $user = User::factory()->has(Character::factory())->create();
    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    actingAs($user);

    $this->post(route('maps.store'), ['name' => ''])->assertSessionHasErrors('name');
});

it('no longer serves a dedicated create-map page', function () {
    $user = User::factory()->has(Character::factory())->create();

    actingAs($user);

    $this->get('/maps/create')->assertNotFound();
});
