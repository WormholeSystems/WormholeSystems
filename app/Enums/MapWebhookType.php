<?php

declare(strict_types=1);

namespace App\Enums;

enum MapWebhookType: string
{
    case Proximity = 'proximity';
    case Killmail = 'killmail';

    public function label(): string
    {
        return match ($this) {
            self::Proximity => 'Known-space connection',
            self::Killmail => 'Killmail in range',
        };
    }
}
