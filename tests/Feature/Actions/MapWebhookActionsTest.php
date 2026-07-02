<?php

declare(strict_types=1);

use App\Actions\MapWebhooks\CreateMapWebhookAction;
use App\Actions\MapWebhooks\DeleteMapWebhookAction;
use App\Actions\MapWebhooks\UpdateMapWebhookAction;
use App\Models\Map;
use App\Models\MapWebhook;

it('creates a webhook for a map', function () {
    $map = Map::factory()->create();

    $webhook = app(CreateMapWebhookAction::class)->handle([
        'map_id' => $map->id,
        'name' => 'Proximity alerts',
        'discord_webhook_url' => 'https://discord.com/api/webhooks/123/abc',
    ]);

    expect($webhook->map_id)->toBe($map->id)
        ->and($webhook->name)->toBe('Proximity alerts')
        ->and(MapWebhook::where('map_id', $map->id)->count())->toBe(1);
});

it('updates a webhook', function () {
    $webhook = MapWebhook::factory()->create(['name' => 'Old']);

    app(UpdateMapWebhookAction::class)->handle($webhook, [
        'name' => 'New',
        'discord_webhook_url' => 'https://discord.com/api/webhooks/456/def',
    ]);

    expect($webhook->fresh()->name)->toBe('New')
        ->and($webhook->fresh()->discord_webhook_url)->toBe('https://discord.com/api/webhooks/456/def');
});

it('deletes a webhook', function () {
    $webhook = MapWebhook::factory()->create();

    app(DeleteMapWebhookAction::class)->handle($webhook);

    expect(MapWebhook::find($webhook->id))->toBeNull();
});
