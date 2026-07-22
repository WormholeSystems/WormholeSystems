<?php

declare(strict_types=1);

namespace App\Enums;

enum MapAlertType: string
{
    case Proximity = 'proximity';
    case Killmail = 'killmail';
    case JumpRange = 'jump_range';

    public function label(): string
    {
        return match ($this) {
            self::Proximity => 'System near chain',
            self::Killmail => 'Kills near chain',
            self::JumpRange => 'Capital jump range',
        };
    }
}
