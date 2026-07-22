<?php

declare(strict_types=1);

namespace App\Enums;

enum MapAlertMentionMode: string
{
    case None = 'none';
    case Creator = 'creator';
    case Role = 'role';
    case Everyone = 'everyone';

    public function label(): string
    {
        return match ($this) {
            self::None => 'No mention',
            self::Creator => 'Alert creator',
            self::Role => 'Discord role',
            self::Everyone => 'Everyone',
        };
    }
}
