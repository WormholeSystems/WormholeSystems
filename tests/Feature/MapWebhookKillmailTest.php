<?php

declare(strict_types=1);

use App\Models\Map;
use App\Models\MapAlert;
use App\Models\MapWebhook;
use App\Models\MapWebhookRole;
use App\Models\User;

use function Pest\Laravel\actingAs;

it('exposes webhooks, roles and killmail alerts on the settings page without the discord url', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create(['name' => 'Killmail channel']);
    MapWebhookRole::factory()->for($map)->create(['name' => 'Fleet ping']);
    MapAlert::factory()->killmail([
        ['subject' => 'alliance', 'side' => 'attacker', 'mode' => 'include', 'ids' => [99000001]],
    ])->create([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
    ]);

    $owner = User::factory()->ownsMap($map)->create();
    $owner->update(['preferred_character_id' => $owner->characters->first()->id]);
    actingAs($owner);

    $this->withoutExceptionHandling()
        ->get(route('maps.settings.discord.show', $map))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page
            ->component('maps/settings/Discord')
            ->where('webhooks.0.name', 'Killmail channel')
            ->missing('webhooks.0.discord_webhook_url')
            ->where('roles.0.name', 'Fleet ping')
            ->where('alerts.0.type', 'killmail')
            ->where('alerts.0.map_webhook_id', $webhook->id)
            ->where('alerts.0.filters.0.subject', 'alliance')
            ->etc()
        );
});
