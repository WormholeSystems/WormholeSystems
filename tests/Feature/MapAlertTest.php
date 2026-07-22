<?php

declare(strict_types=1);

use App\Enums\JumpShipType;
use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertEventAction;
use App\Enums\MapAlertMentionMode;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapAlert;
use App\Models\MapWebhook;
use App\Models\MapWebhookRole;
use App\Models\User;

use function Pest\Laravel\actingAs;

function validAlertPayload(Map $map, MapWebhook $webhook, array $overrides = []): array
{
    return array_merge([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
        'type' => 'proximity',
        'target_solarsystem_id' => makeSolarsystem(30009300),
        'max_jumps' => 5,
        'is_active' => true,
    ], $overrides);
}

function mapAlertManager(Map $map, Permission $permission): User
{
    return User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();
}

it('lets a manager create a proximity alert pointing at a webhook and role', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $role = MapWebhookRole::factory()->for($map)->create();
    $manager = mapAlertManager($map, Permission::Manager);
    actingAs($manager);

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, [
        'map_webhook_role_id' => $role->id,
        'created_by_user_id' => 999999,
        'delivery_type' => MapAlertDeliveryType::DiscordDm->value,
    ]))
        ->assertRedirect();

    $alert = MapAlert::query()->where('map_id', $map->id)->sole();

    expect($alert->map_webhook_id)->toBe($webhook->id)
        ->and($alert->map_webhook_role_id)->toBe($role->id)
        ->and($alert->type->value)->toBe('proximity')
        ->and($alert->created_by_user_id)->toBe($manager->id)
        ->and($alert->delivery_type)->toBe(MapAlertDeliveryType::Webhook)
        ->and($alert->events()->where('action', MapAlertEventAction::Created)->count())->toBe(1)
        ->and($alert->events()->sole()->actor_user_id)->toBe($manager->id);
});

it('creates an inactive webhook alert with disabled metadata and lifecycle events', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $manager = mapAlertManager($map, Permission::Manager);
    actingAs($manager);

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, ['is_active' => false]))
        ->assertRedirect();

    $alert = MapAlert::query()->where('map_id', $map->id)->sole();

    expect($alert->is_active)->toBeFalse()
        ->and($alert->disabled_at)->not->toBeNull()
        ->and($alert->disabled_by_user_id)->toBe($manager->id)
        ->and($alert->disabled_reason)->toBe(MapAlertDisabledReason::Manual)
        ->and($alert->events()->pluck('action')->all())->toBe([
            MapAlertEventAction::Created,
            MapAlertEventAction::Disabled,
        ]);
});

it('lets a manager create a killmail alert with filters and a null target', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $manager = mapAlertManager($map, Permission::Manager);
    actingAs($manager);

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, [
        'type' => 'killmail',
        'target_solarsystem_id' => null,
        'filter_match' => 'any',
        'filters' => [
            ['subject' => 'corporation', 'side' => 'either', 'mode' => 'include', 'ids' => [98000001, 98000002]],
        ],
    ]))->assertRedirect();

    $alert = MapAlert::query()->where('map_id', $map->id)->sole();

    expect($alert->type->value)->toBe('killmail')
        ->and($alert->target_solarsystem_id)->toBeNull()
        ->and($alert->filters)->toHaveCount(1)
        ->and($alert->filters->first()->ids)->toBe([98000001, 98000002]);
});

it('lets a manager create a jump-range alert', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, [
        'type' => 'jump_range',
        'max_jumps' => null,
        'ship_type' => 'dreadnought',
        'jdc_level' => 5,
        'include_highsec' => true,
    ]))->assertRedirect();

    $alert = MapAlert::query()->where('map_id', $map->id)->sole();

    expect($alert->type->value)->toBe('jump_range')
        ->and($alert->ship_type)->toBe(JumpShipType::Dreadnought)
        ->and($alert->jdc_level)->toBe(5)
        ->and($alert->include_highsec)->toBeTrue()
        ->and($alert->max_jumps)->toBeNull();
});

it('validates jump-range alert fields', function (array $overrides, string $invalidField) {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, array_merge([
        'type' => 'jump_range',
        'max_jumps' => null,
        'ship_type' => 'dreadnought',
        'jdc_level' => 5,
    ], $overrides)))->assertInvalid([$invalidField]);
})->with([
    'missing target' => [['target_solarsystem_id' => null], 'target_solarsystem_id'],
    'missing ship' => [['ship_type' => null], 'ship_type'],
    'invalid ship' => [['ship_type' => 'battleship'], 'ship_type'],
    'missing jdc' => [['jdc_level' => null], 'jdc_level'],
    'jdc too low' => [['jdc_level' => 0], 'jdc_level'],
    'jdc too high' => [['jdc_level' => 6], 'jdc_level'],
]);

it('still requires max jumps for non jump-range alerts', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, ['max_jumps' => null]))
        ->assertInvalid(['max_jumps']);
});

it('requires a target for proximity alerts', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, ['target_solarsystem_id' => null]))
        ->assertInvalid(['target_solarsystem_id']);
});

it('rejects a webhook that belongs to another map', function () {
    $map = Map::factory()->create();
    $otherWebhook = MapWebhook::factory()->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $otherWebhook))
        ->assertInvalid(['map_webhook_id']);
});

it('rejects a role that belongs to another map', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $otherRole = MapWebhookRole::factory()->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, ['map_webhook_role_id' => $otherRole->id]))
        ->assertInvalid(['map_webhook_role_id']);
});

it('validates filter rules', function (array $filter, string $invalidField) {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, [
        'type' => 'killmail',
        'target_solarsystem_id' => null,
        'filters' => [$filter],
    ]))->assertInvalid([$invalidField]);
})->with([
    'bad subject' => [['subject' => 'wallet', 'side' => 'either', 'mode' => 'include', 'ids' => [1]], 'filters.0.subject'],
    'bad side' => [['subject' => 'character', 'side' => 'nobody', 'mode' => 'include', 'ids' => [1]], 'filters.0.side'],
    'bad mode' => [['subject' => 'character', 'side' => 'either', 'mode' => 'maybe', 'ids' => [1]], 'filters.0.mode'],
    'non-integer id' => [['subject' => 'character', 'side' => 'either', 'mode' => 'include', 'ids' => ['abc']], 'filters.0.ids.0'],
    'empty ids' => [['subject' => 'character', 'side' => 'either', 'mode' => 'include', 'ids' => []], 'filters.0.ids'],
]);

it('lets a manager update an alert', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $alert = MapAlert::factory()->create([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
        'target_solarsystem_id' => makeSolarsystem(30009301),
        'max_jumps' => 5,
    ]);
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->put(route('map-alerts.update', $alert), [
        'map_webhook_id' => $webhook->id,
        'type' => 'proximity',
        'target_solarsystem_id' => $alert->target_solarsystem_id,
        'max_jumps' => 12,
        'is_active' => false,
    ])->assertRedirect();

    expect($alert->refresh()->max_jumps)->toBe(12)
        ->and($alert->is_active)->toBeFalse()
        ->and($alert->disabled_at)->not->toBeNull()
        ->and($alert->disabled_reason)->toBe(MapAlertDisabledReason::Manual)
        ->and($alert->events()->pluck('action')->all())->toBe([
            MapAlertEventAction::Updated,
            MapAlertEventAction::Disabled,
        ]);
});

it('lets a manager delete an alert', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $alert = MapAlert::factory()->create([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
        'target_solarsystem_id' => makeSolarsystem(30009302),
    ]);
    $manager = mapAlertManager($map, Permission::Manager);
    actingAs($manager);

    $this->delete(route('map-alerts.destroy', $alert))->assertRedirect();

    $deletedAlert = MapAlert::withTrashed()->findOrFail($alert->id);
    $event = $deletedAlert->events()->sole();

    expect(MapAlert::query()->whereKey($alert->id)->exists())->toBeFalse()
        ->and($deletedAlert->deleted_at)->not->toBeNull()
        ->and($deletedAlert->deleted_by_user_id)->toBe($manager->id)
        ->and($event->action)->toBe(MapAlertEventAction::Removed)
        ->and($event->map_id)->toBe($map->id)
        ->and($event->actor_user_id)->toBe($manager->id)
        ->and($event->snapshot['id'])->toBe($alert->id);
});

it('forbids a viewer from creating an alert', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(mapAlertManager($map, Permission::Viewer));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook))->assertForbidden();

    expect(MapAlert::query()->where('map_id', $map->id)->count())->toBe(0);
});

it('lets a manager create an alert that pings everyone', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, [
        'mention_mode' => 'everyone',
        'map_webhook_role_id' => null,
    ]))->assertRedirect();

    $alert = MapAlert::query()->where('map_id', $map->id)->sole();

    expect($alert->mention_mode)->toBe(MapAlertMentionMode::Everyone)
        ->and($alert->map_webhook_role_id)->toBeNull();
});

it('rejects an everyone mention combined with a mention record', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $role = MapWebhookRole::factory()->for($map)->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, [
        'mention_mode' => 'everyone',
        'map_webhook_role_id' => $role->id,
    ]))->assertInvalid(['map_webhook_role_id']);
});

it('rejects bot-only mention modes for webhook alerts', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(mapAlertManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, [
        'mention_mode' => 'creator',
    ]))->assertInvalid(['mention_mode']);
});
