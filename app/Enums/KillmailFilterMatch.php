<?php

declare(strict_types=1);

namespace App\Enums;

/**
 * How a killmail webhook's include filters combine: match any one of them (OR) or
 * require all of them (AND). Exclude filters always veto regardless of this mode.
 */
enum KillmailFilterMatch: string
{
    case Any = 'any';
    case All = 'all';

    public function label(): string
    {
        return match ($this) {
            self::Any => 'Match any filter',
            self::All => 'Match all filters',
        };
    }
}
