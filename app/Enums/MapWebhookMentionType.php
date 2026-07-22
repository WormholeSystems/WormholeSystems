<?php

declare(strict_types=1);

namespace App\Enums;

enum MapWebhookMentionType: string
{
    case Role = 'role';
    case User = 'user';

    public function label(): string
    {
        return match ($this) {
            self::Role => 'Role',
            self::User => 'User',
        };
    }
}
