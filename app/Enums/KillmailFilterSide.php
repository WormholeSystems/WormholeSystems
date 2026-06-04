<?php

declare(strict_types=1);

namespace App\Enums;

/**
 * Which participant of a killmail a filter rule matches against.
 */
enum KillmailFilterSide: string
{
    case Victim = 'victim';
    case Attacker = 'attacker';
    case Either = 'either';

    public function label(): string
    {
        return match ($this) {
            self::Victim => 'Victim',
            self::Attacker => 'Attacker',
            self::Either => 'Either side',
        };
    }
}
