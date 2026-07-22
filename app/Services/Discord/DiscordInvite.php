<?php

declare(strict_types=1);

namespace App\Services\Discord;

final class DiscordInvite
{
    private const int PERMISSIONS = 19456;

    public function url(): ?string
    {
        $applicationId = config('services.discord.application_id');
        if (! is_string($applicationId) || $applicationId === '') {
            return null;
        }

        return 'https://discord.com/oauth2/authorize?'.http_build_query([
            'client_id' => $applicationId,
            'scope' => 'bot applications.commands',
            'permissions' => self::PERMISSIONS,
            'integration_type' => 0,
        ], encoding_type: PHP_QUERY_RFC3986);
    }
}
