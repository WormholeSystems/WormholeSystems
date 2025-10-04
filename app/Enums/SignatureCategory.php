<?php

declare(strict_types=1);

namespace App\Enums;

enum SignatureCategory: string
{
    case Wormhole = 'wormhole';
    case Data = 'data';
    case Relic = 'relic';
    case Combat = 'combat';
    case Gas = 'gas';
    case Ore = 'ore';

    public function name(): string
    {
        return match ($this) {
            self::Wormhole => 'Wormhole',
            self::Data => 'Data Site',
            self::Relic => 'Relic Site',
            self::Combat => 'Combat Site',
            self::Gas => 'Gas Site',
            self::Ore => 'Ore Site',
        };
    }
}
