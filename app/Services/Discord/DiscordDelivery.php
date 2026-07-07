<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Models\MapAlert;
use Illuminate\Http\Client\RequestException;
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

    private function wasRateLimited(Throwable $exception): bool
    {
        return $exception instanceof RequestException && $exception->response->status() === 429;
    }

    private function retryDelayMilliseconds(Throwable $exception): int
    {
        if (! $exception instanceof RequestException) {
            return 200;
        }

        $retryAfterSeconds = (float) $exception->response->header('Retry-After');

        return $retryAfterSeconds > 0 ? (int) ceil($retryAfterSeconds * 1000) : 200;
    }
}
