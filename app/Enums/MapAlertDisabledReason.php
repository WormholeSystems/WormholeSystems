<?php

declare(strict_types=1);

namespace App\Enums;

enum MapAlertDisabledReason: string
{
    case Manual = 'manual';
    case DiscordAccountDisconnected = 'discord_account_disconnected';
    case AccessRevoked = 'access_revoked';
    case DiscordDestinationUnavailable = 'discord_destination_unavailable';
    case DeliveryFailed = 'delivery_failed';

    public function label(): string
    {
        return match ($this) {
            self::Manual => 'Disabled by a manager',
            self::DiscordAccountDisconnected => 'Discord account disconnected',
            self::AccessRevoked => 'Map access revoked',
            self::DiscordDestinationUnavailable => 'Discord destination unavailable',
            self::DeliveryFailed => 'Delivery failed',
        };
    }
}
