<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\MapWebhook;
use App\Models\MapWebhookRole;

use function Pest\Laravel\actingAs;

it('lets a manager create a role', function () {
    $map = Map::factory()->create();
    actingAs(webhookManager($map, Permission::Manager));

    $this->post(route('map-webhook-roles.store'), [
        'map_id' => $map->id,
        'name' => 'Ping squad',
        'discord_role_id' => '987654321',
    ])->assertRedirect();

    expect(MapWebhookRole::query()->where('map_id', $map->id)->sole()->discord_role_id)->toBe('987654321');
});

it('rejects a non-numeric role id', function () {
    $map = Map::factory()->create();
    actingAs(webhookManager($map, Permission::Manager));

    $this->post(route('map-webhook-roles.store'), [
        'map_id' => $map->id,
        'name' => 'Ping squad',
        'discord_role_id' => 'not-a-role',
    ])->assertInvalid(['discord_role_id']);
});

it('lets a manager update a role', function () {
    $map = Map::factory()->create();
    $role = MapWebhookRole::factory()->for($map)->create(['name' => 'Old']);
    actingAs(webhookManager($map, Permission::Manager));

    $this->put(route('map-webhook-roles.update', $role), ['name' => 'New', 'discord_role_id' => '555'])->assertRedirect();

    expect($role->refresh()->name)->toBe('New')
        ->and($role->discord_role_id)->toBe('555');
});

it('detaches the role from its alerts when deleted, without deleting them', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $role = MapWebhookRole::factory()->for($map)->create();
    $alert = MapAlert::factory()->create([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
        'map_webhook_role_id' => $role->id,
        'target_solarsystem_id' => makeSolarsystem(30009130),
    ]);
    actingAs(webhookManager($map, Permission::Manager));

    $this->delete(route('map-webhook-roles.destroy', $role))->assertRedirect();

    expect(MapWebhookRole::query()->whereKey($role->id)->exists())->toBeFalse()
        ->and($alert->refresh()->map_webhook_role_id)->toBeNull();
});

it('forbids a member from creating a role', function () {
    $map = Map::factory()->create();
    actingAs(webhookManager($map, Permission::Member));

    $this->post(route('map-webhook-roles.store'), [
        'map_id' => $map->id,
        'name' => 'Ping squad',
        'discord_role_id' => '987654321',
    ])->assertForbidden();

    expect(MapWebhookRole::query()->where('map_id', $map->id)->count())->toBe(0);
});
