<?php

declare(strict_types=1);

namespace App\Enums;

enum MapAlertDeliveryType: string
{
    case Webhook = 'webhook';
    case DiscordDm = 'discord_dm';
    case DiscordChannel = 'discord_channel';

    /** @return list<self> */
    public static function botTypes(): array
    {
        return array_values(array_filter(self::cases(), fn (self $type): bool => $type->isBot()));
    }

    /** @return list<self> */
    public static function sharedTypes(): array
    {
        return array_values(array_filter(self::cases(), fn (self $type): bool => $type->isShared()));
    }

    public function label(): string
    {
        return match ($this) {
            self::Webhook => 'Webhook',
            self::DiscordDm => 'Private Discord message',
            self::DiscordChannel => 'Discord channel',
        };
    }

    public function isBot(): bool
    {
        return $this !== self::Webhook;
    }

    public function isShared(): bool
    {
        return $this !== self::DiscordDm;
    }
}
