<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapWebhook;
use App\Models\User;

use function Pest\Laravel\actingAs;

function killmailManager(Map $map): User
{
    return User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Manager])->for($map)))
        ->create();
}

function killmailWebhookPayload(Map $map, array $overrides = []): array
{
    return array_merge([
        'map_id' => $map->id,
        'name' => 'Hostiles nearby',
        'discord_webhook_url' => 'https://discord.com/api/webhooks/123456789/abcdefg',
        'type' => 'killmail',
        'max_jumps' => 5,
        'filters' => [
            ['subject' => 'corporation', 'side' => 'either', 'mode' => 'include', 'ids' => [98000001, 98000002]],
        ],
        'is_active' => true,
    ], $overrides);
}

it('creates a killmail webhook with filters and a null target', function () {
    $map = Map::factory()->create();
    actingAs(killmailManager($map));

    $this->post(route('map-webhooks.store'), killmailWebhookPayload($map))->assertRedirect();

    $webhook = MapWebhook::query()->where('map_id', $map->id)->sole();

    expect($webhook->type->value)->toBe('killmail')
        ->and($webhook->target_solarsystem_id)->toBeNull()
        ->and($webhook->filters)->toHaveCount(1)
        ->and($webhook->filters->first()->ids)->toBe([98000001, 98000002]);
});

it('persists a discord role id and rejects a non-numeric one', function () {
    $map = Map::factory()->create();
    actingAs(killmailManager($map));

    $this->post(route('map-webhooks.store'), killmailWebhookPayload($map, ['discord_role_id' => '987654321']))->assertRedirect();
    expect(MapWebhook::query()->where('map_id', $map->id)->sole()->discord_role_id)->toBe('987654321');

    $this->post(route('map-webhooks.store'), killmailWebhookPayload($map, ['discord_role_id' => 'not-a-role']))->assertInvalid(['discord_role_id']);
});

it('allows a killmail webhook without a target system', function () {
    $map = Map::factory()->create();
    actingAs(killmailManager($map));

    $this->post(route('map-webhooks.store'), killmailWebhookPayload($map, ['filters' => []]))->assertValid();
});

it('requires a target for proximity webhooks but not killmail webhooks', function () {
    $map = Map::factory()->create();
    actingAs(killmailManager($map));

    $this->post(route('map-webhooks.store'), killmailWebhookPayload($map, ['type' => 'proximity', 'target_solarsystem_id' => null]))
        ->assertInvalid(['target_solarsystem_id']);
});

it('validates filter rules', function (array $filter, string $invalidField) {
    $map = Map::factory()->create();
    actingAs(killmailManager($map));

    $this->post(route('map-webhooks.store'), killmailWebhookPayload($map, ['filters' => [$filter]]))
        ->assertInvalid([$invalidField]);
})->with([
    'bad subject' => [['subject' => 'wallet', 'side' => 'either', 'mode' => 'include', 'ids' => [1]], 'filters.0.subject'],
    'bad side' => [['subject' => 'character', 'side' => 'nobody', 'mode' => 'include', 'ids' => [1]], 'filters.0.side'],
    'bad mode' => [['subject' => 'character', 'side' => 'either', 'mode' => 'maybe', 'ids' => [1]], 'filters.0.mode'],
    'non-integer id' => [['subject' => 'character', 'side' => 'either', 'mode' => 'include', 'ids' => ['abc']], 'filters.0.ids.0'],
    'empty ids' => [['subject' => 'character', 'side' => 'either', 'mode' => 'include', 'ids' => []], 'filters.0.ids'],
]);

it('exposes filters but never the discord url on the settings page', function () {
    $map = Map::factory()->create();
    MapWebhook::factory()->for($map)->killmail([
        ['subject' => 'alliance', 'side' => 'attacker', 'mode' => 'include', 'ids' => [99000001]],
    ])->create(['name' => 'Killmail keeper']);

    $owner = User::factory()->ownsMap($map)->create();
    $owner->update(['preferred_character_id' => $owner->characters->first()->id]);
    actingAs($owner);

    $this->withoutExceptionHandling()
        ->get(route('maps.settings.webhooks.show', $map))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page
            ->component('maps/settings/ShowWebhooks')
            ->where('webhooks.0.name', 'Killmail keeper')
            ->where('webhooks.0.type', 'killmail')
            ->where('webhooks.0.filters.0.subject', 'alliance')
            ->missing('webhooks.0.discord_webhook_url')
            ->etc()
        );
});
