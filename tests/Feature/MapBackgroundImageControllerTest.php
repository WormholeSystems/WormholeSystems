<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;
use Illuminate\Http\UploadedFile;
use Illuminate\Support\Facades\Storage;

use function Pest\Laravel\actingAs;

function userWithMapAccess(Map $map): User
{
    return User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))
        ->create();
}

it('stores an uploaded background image and persists the path', function () {
    Storage::fake('public');

    $map = Map::factory()->create();
    $user = userWithMapAccess($map);

    actingAs($user);

    $this->post(route('maps.background-image.store', $map), [
        'background_image' => UploadedFile::fake()->image('background.png', 1920, 1080),
    ])->assertRedirect();

    $settings = $user->mapUserSettings()->where('map_id', $map->id)->first();

    expect($settings->background_image_path)->not->toBeNull()
        ->and($settings->background_image_path)->toStartWith("map-backgrounds/{$map->id}/");

    Storage::disk('public')->assertExists($settings->background_image_path);
});

it('exposes the stored image as a url in the settings resource', function () {
    Storage::fake('public');

    $map = Map::factory()->create();
    $user = User::factory()->ownsMap($map)->create();
    $user->update(['preferred_character_id' => $user->characters->first()->id]);

    actingAs($user);

    $this->post(route('maps.background-image.store', $map), [
        'background_image' => UploadedFile::fake()->image('background.png'),
    ])->assertRedirect();

    $this->get(route('maps.show', $map))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page
            ->has('map_user_settings', fn ($settings) => $settings
                ->where('background_image_url', fn ($url) => is_string($url) && str_contains($url, '/storage/map-backgrounds/'))
                ->etc()
            )
        );
});

it('deletes the previous image when a new one is uploaded', function () {
    Storage::fake('public');

    $map = Map::factory()->create();
    $user = userWithMapAccess($map);

    actingAs($user);

    $this->post(route('maps.background-image.store', $map), [
        'background_image' => UploadedFile::fake()->image('first.png'),
    ])->assertRedirect();

    $firstPath = $user->mapUserSettings()->where('map_id', $map->id)->first()->background_image_path;

    $this->post(route('maps.background-image.store', $map), [
        'background_image' => UploadedFile::fake()->image('second.png'),
    ])->assertRedirect();

    $secondPath = $user->mapUserSettings()->where('map_id', $map->id)->first()->background_image_path;

    expect($secondPath)->not->toBe($firstPath);
    Storage::disk('public')->assertMissing($firstPath);
    Storage::disk('public')->assertExists($secondPath);
});

it('removes the background image and clears the path', function () {
    Storage::fake('public');

    $map = Map::factory()->create();
    $user = userWithMapAccess($map);

    actingAs($user);

    $this->post(route('maps.background-image.store', $map), [
        'background_image' => UploadedFile::fake()->image('background.png'),
    ])->assertRedirect();

    $path = $user->mapUserSettings()->where('map_id', $map->id)->first()->background_image_path;

    $this->delete(route('maps.background-image.destroy', $map))->assertRedirect();

    expect($user->mapUserSettings()->where('map_id', $map->id)->first()->background_image_path)->toBeNull();
    Storage::disk('public')->assertMissing($path);
});

it('rejects a non-image upload', function () {
    Storage::fake('public');

    $map = Map::factory()->create();
    $user = userWithMapAccess($map);

    actingAs($user);

    $this->post(route('maps.background-image.store', $map), [
        'background_image' => UploadedFile::fake()->create('document.pdf', 100, 'application/pdf'),
    ])->assertSessionHasErrors('background_image');

    expect($user->mapUserSettings()->where('map_id', $map->id)->first()?->background_image_path)->toBeNull();
});

it('rejects an image larger than 8 MB', function () {
    Storage::fake('public');

    $map = Map::factory()->create();
    $user = userWithMapAccess($map);

    actingAs($user);

    $this->post(route('maps.background-image.store', $map), [
        'background_image' => UploadedFile::fake()->image('huge.png')->size(8193),
    ])->assertSessionHasErrors('background_image');

    expect($user->mapUserSettings()->where('map_id', $map->id)->first()?->background_image_path)->toBeNull();
});

it('forbids an authenticated user without access from uploading', function () {
    Storage::fake('public');

    $map = Map::factory()->create();
    $user = User::factory()->has(Character::factory())->create();

    actingAs($user);

    $this->post(route('maps.background-image.store', $map), [
        'background_image' => UploadedFile::fake()->image('background.png'),
    ])->assertForbidden();
});

it('requires authentication to upload a background image', function () {
    Storage::fake('public');

    $map = Map::factory()->create(['is_public' => true]);

    $this->post(route('maps.background-image.store', $map), [
        'background_image' => UploadedFile::fake()->image('background.png'),
    ])->assertRedirect(route('login'));
});
