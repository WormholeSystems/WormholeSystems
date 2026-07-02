<?php

declare(strict_types=1);

use App\Enums\JumpShipType;
use App\Enums\Permission;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\MapWebhook;
use App\Models\MapWebhookRole;

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

it('lets a manager create a proximity alert pointing at a webhook and role', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $role = MapWebhookRole::factory()->for($map)->create();
    actingAs(webhookManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, ['map_webhook_role_id' => $role->id]))
        ->assertRedirect();

    $alert = MapAlert::query()->where('map_id', $map->id)->sole();

    expect($alert->map_webhook_id)->toBe($webhook->id)
        ->and($alert->map_webhook_role_id)->toBe($role->id)
        ->and($alert->type->value)->toBe('proximity');
});

it('lets a manager create a killmail alert with filters and a null target', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(webhookManager($map, Permission::Manager));

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
    actingAs(webhookManager($map, Permission::Manager));

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
    actingAs(webhookManager($map, Permission::Manager));

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
    actingAs(webhookManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, ['max_jumps' => null]))
        ->assertInvalid(['max_jumps']);
});

it('requires a target for proximity alerts', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(webhookManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, ['target_solarsystem_id' => null]))
        ->assertInvalid(['target_solarsystem_id']);
});

it('rejects a webhook that belongs to another map', function () {
    $map = Map::factory()->create();
    $otherWebhook = MapWebhook::factory()->create();
    actingAs(webhookManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $otherWebhook))
        ->assertInvalid(['map_webhook_id']);
});

it('rejects a role that belongs to another map', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $otherRole = MapWebhookRole::factory()->create();
    actingAs(webhookManager($map, Permission::Manager));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook, ['map_webhook_role_id' => $otherRole->id]))
        ->assertInvalid(['map_webhook_role_id']);
});

it('validates filter rules', function (array $filter, string $invalidField) {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(webhookManager($map, Permission::Manager));

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
    actingAs(webhookManager($map, Permission::Manager));

    $this->put(route('map-alerts.update', $alert), [
        'map_webhook_id' => $webhook->id,
        'type' => 'proximity',
        'target_solarsystem_id' => $alert->target_solarsystem_id,
        'max_jumps' => 12,
        'is_active' => true,
    ])->assertRedirect();

    expect($alert->refresh()->max_jumps)->toBe(12);
});

it('lets a manager delete an alert', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $alert = MapAlert::factory()->create([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
        'target_solarsystem_id' => makeSolarsystem(30009302),
    ]);
    actingAs(webhookManager($map, Permission::Manager));

    $this->delete(route('map-alerts.destroy', $alert))->assertRedirect();

    expect(MapAlert::query()->whereKey($alert->id)->exists())->toBeFalse();
});

it('forbids a viewer from creating an alert', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    actingAs(webhookManager($map, Permission::Viewer));

    $this->post(route('map-alerts.store'), validAlertPayload($map, $webhook))->assertForbidden();

    expect(MapAlert::query()->where('map_id', $map->id)->count())->toBe(0);
});
