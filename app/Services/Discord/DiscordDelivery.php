<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Enums\MapAlertMentionMode;
use App\Models\MapAlert;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Throwable;

/**
 * POSTs a prepared embed to an alert's Discord webhook, pinging the alert's role when one
 * is set. Delivery failures are logged but never bubble up so a bad webhook can't break
 * the surrounding job.
 *
 * The POST is not idempotent: on a timeout or 5xx Discord may already have accepted the
 * message, so blindly retrying posts the same alert twice. Only 429 rate limits are
 * retried — there Discord guarantees the request was rejected — honoring Retry-After.
 */
final readonly class DiscordDelivery
{
    use RetriesDiscordRateLimits;

    /**
     * @param  array<string, mixed>  $embed
     */
    public function deliver(MapAlert $alert, array $embed): void
    {
        $payload = ['embeds' => [$embed]];

        if ($alert->mention_mode === MapAlertMentionMode::Everyone) {
            $payload['content'] = '@everyone';
            $payload['allowed_mentions'] = ['parse' => ['everyone']];
        } elseif ($alert->role !== null) {
            $mention = $alert->role->mention();
            $payload['content'] = $mention['content'];
            $payload['allowed_mentions'] = $mention['allowed_mentions'];
        }

        try {
            Http::timeout(10)
                ->retry(
                    3,
                    fn (int $attempt, Throwable $exception): int => $this->retryDelayMilliseconds($exception),
                    fn (Throwable $exception): bool => $this->wasRateLimited($exception),
                )
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
