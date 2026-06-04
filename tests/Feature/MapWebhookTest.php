<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapWebhook;
use App\Models\User;

use function Pest\Laravel\actingAs;

function webhookManager(Map $map, Permission $permission): User
{
    return User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();
}

function validWebhookPayload(Map $map, array $overrides = []): array
{
    return array_merge([
        'map_id' => $map->id,
        'name' => 'Jita alert',
        'discord_webhook_url' => 'https://discord.com/api/webhooks/123456789/abcdefg',
        'type' => 'proximity',
        'target_solarsystem_id' => makeSolarsystem(30009100),
        'max_jumps' => 5,
        'is_active' => true,
    ], $overrides);
}

it('lets a manager create a webhook', function () {
    $map = Map::factory()->create();
    actingAs(webhookManager($map, Permission::Manager));

    $this->post(route('map-webhooks.store'), validWebhookPayload($map))->assertRedirect();

    expect(MapWebhook::query()->where('map_id', $map->id)->count())->toBe(1);
});

it('lets a manager update a webhook', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create([
        'name' => 'Old',
        'max_jumps' => 5,
        'target_solarsystem_id' => makeSolarsystem(30009101),
    ]);
    actingAs(webhookManager($map, Permission::Manager));

    $this->put(route('map-webhooks.update', $webhook), [
        'name' => 'New',
        'type' => 'proximity',
        'target_solarsystem_id' => $webhook->target_solarsystem_id,
        'max_jumps' => 10,
        'is_active' => true,
    ])->assertRedirect();

    expect($webhook->refresh()->name)->toBe('New')
        ->and($webhook->max_jumps)->toBe(10);
});

it('lets a manager delete a webhook', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create(['target_solarsystem_id' => makeSolarsystem(30009103)]);
    actingAs(webhookManager($map, Permission::Manager));

    $this->delete(route('map-webhooks.destroy', $webhook))->assertRedirect();

    expect(MapWebhook::query()->whereKey($webhook->id)->exists())->toBeFalse();
});

it('forbids a member from creating a webhook', function () {
    $map = Map::factory()->create();
    actingAs(webhookManager($map, Permission::Member));

    $this->post(route('map-webhooks.store'), validWebhookPayload($map))->assertForbidden();

    expect(MapWebhook::query()->where('map_id', $map->id)->count())->toBe(0);
});

it('forbids a viewer from deleting a webhook', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create(['target_solarsystem_id' => makeSolarsystem(30009104)]);
    actingAs(webhookManager($map, Permission::Viewer));

    $this->delete(route('map-webhooks.destroy', $webhook))->assertForbidden();

    expect(MapWebhook::query()->whereKey($webhook->id)->exists())->toBeTrue();
});

it('validates the webhook payload', function (array $overrides, string $invalidField) {
    $map = Map::factory()->create();
    actingAs(webhookManager($map, Permission::Manager));

    $this->post(route('map-webhooks.store'), validWebhookPayload($map, $overrides))
        ->assertInvalid([$invalidField]);
})->with([
    'malformed url' => [['discord_webhook_url' => 'not-a-url'], 'discord_webhook_url'],
    'non-discord url' => [['discord_webhook_url' => 'https://example.com/hook'], 'discord_webhook_url'],
    'max jumps too high' => [['max_jumps' => 99], 'max_jumps'],
    'missing target' => [['target_solarsystem_id' => 999999999], 'target_solarsystem_id'],
]);

it('never exposes the discord url on the settings page', function () {
    $map = Map::factory()->create();
    MapWebhook::factory()->for($map)->create([
        'name' => 'Secret keeper',
        'target_solarsystem_id' => makeSolarsystem(30009105),
    ]);
    $owner = User::factory()->ownsMap($map)->create();
    $owner->update(['preferred_character_id' => $owner->characters->first()->id]);
    actingAs($owner);

    $this->withoutExceptionHandling()
        ->get(route('maps.settings.webhooks.show', $map))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page
            ->component('maps/settings/ShowWebhooks')
            ->where('webhooks.0.name', 'Secret keeper')
            ->missing('webhooks.0.discord_webhook_url')
            ->etc()
        );
});
