<?php

declare(strict_types=1);

namespace App\Services\Discord;

use Illuminate\Http\Client\RequestException;
use Throwable;

/**
 * Retry policy for Discord 429 responses: only rate limits are retried (Discord guarantees
 * the request was rejected), honoring the Retry-After header when present.
 */
trait RetriesDiscordRateLimits
{
    private const int RETRY_FALLBACK_MILLISECONDS = 200;

    private function wasRateLimited(Throwable $exception): bool
    {
        return $exception instanceof RequestException && $exception->response->status() === 429;
    }

    private function retryDelayMilliseconds(Throwable $exception): int
    {
        if (! $exception instanceof RequestException) {
            return self::RETRY_FALLBACK_MILLISECONDS;
        }

        $retryAfterSeconds = (float) $exception->response->header('Retry-After');

        return $retryAfterSeconds > 0 ? (int) ceil($retryAfterSeconds * 1000) : self::RETRY_FALLBACK_MILLISECONDS;
    }
}
