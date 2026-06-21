<?php

declare(strict_types=1);

use App\Actions\MapWebhooks\CreateMapWebhookAction;
use App\Actions\MapWebhooks\DeleteMapWebhookAction;
use App\Actions\MapWebhooks\UpdateMapWebhookAction;
use App\Enums\MapWebhookType;
use App\Models\Map;
use App\Models\MapWebhook;

it('creates a webhook for a map', function () {
    $map = Map::factory()->create();

    $webhook = app(CreateMapWebhookAction::class)->handle([
        'map_id' => $map->id,
        'name' => 'Proximity alerts',
        'discord_webhook_url' => 'https://discord.com/api/webhooks/123/abc',
        'type' => MapWebhookType::Proximity,
        'target_solarsystem_id' => makeSolarsystem(30010001),
        'max_jumps' => 5,
        'is_active' => true,
    ]);

    expect($webhook->map_id)->toBe($map->id)
        ->and($webhook->name)->toBe('Proximity alerts')
        ->and(MapWebhook::where('map_id', $map->id)->count())->toBe(1);
});

it('updates a webhook', function () {
    makeSolarsystem(30010002);
    $webhook = MapWebhook::factory()->create(['name' => 'Old', 'is_active' => true]);

    app(UpdateMapWebhookAction::class)->handle($webhook, ['name' => 'New', 'is_active' => false]);

    expect($webhook->fresh()->name)->toBe('New')
        ->and($webhook->fresh()->is_active)->toBeFalse();
});

it('deletes a webhook', function () {
    makeSolarsystem(30010003);
    $webhook = MapWebhook::factory()->create();

    app(DeleteMapWebhookAction::class)->handle($webhook);

    expect(MapWebhook::find($webhook->id))->toBeNull();
});
