<?php

declare(strict_types=1);

namespace App\Services\Discord;

use Illuminate\Http\Client\PendingRequest;
use Illuminate\Support\Facades\Http;
use Throwable;

/**
 * Single place the Discord REST API client is configured: base URL, bot authentication,
 * timeout, and the 429 retry policy.
 */
final class DiscordHttp
{
    use RetriesDiscordRateLimits;

    public const string BASE_URL = 'https://discord.com/api/v10';

    public function request(): PendingRequest
    {
        return Http::baseUrl(self::BASE_URL)
            ->acceptJson()
            ->retry(
                3,
                fn (int $attempt, Throwable $exception): int => $this->retryDelayMilliseconds($exception),
                fn (Throwable $exception): bool => $this->wasRateLimited($exception),
            )
            ->timeout(10);
    }

    public function bot(): PendingRequest
    {
        $token = config('services.discord.bot_token');
        if (! is_string($token) || $token === '') {
            throw new DiscordConfigurationException('Discord bot token is not configured.');
        }

        return $this->request()->withHeaders(['Authorization' => 'Bot '.$token]);
    }
}
