<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Models\MapAlert;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Throwable;

/**
 * POSTs a prepared embed to an alert's Discord webhook, pinging the alert's role when one
 * is set. Delivery failures are logged but never bubble up so a bad webhook can't break
 * the surrounding job.
 */
final readonly class DiscordDelivery
{
    /**
     * @param  array<string, mixed>  $embed
     */
    public function deliver(MapAlert $alert, array $embed): void
    {
        $payload = ['embeds' => [$embed]];

        $roleId = $alert->role?->discord_role_id;

        if ($roleId !== null) {
            $payload['content'] = sprintf('<@&%s>', $roleId);
            $payload['allowed_mentions'] = ['roles' => [$roleId]];
        }

        try {
            Http::timeout(10)
                ->retry(3, 200)
                ->post($alert->webhook->discord_webhook_url, $payload)
                ->throw();
        } catch (Throwable $e) {
            Log::warning('Discord webhook delivery failed', [
                'map_alert_id' => $alert->id,
                'message' => $e->getMessage(),
            ]);
        }
    }
}
