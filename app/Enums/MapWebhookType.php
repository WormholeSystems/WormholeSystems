<?php

declare(strict_types=1);

namespace App\Enums;

enum MapWebhookType: string
{
    case Proximity = 'proximity';

    public function label(): string
    {
        return match ($this) {
            self::Proximity => 'Known-space connection',
        };
    }
}
