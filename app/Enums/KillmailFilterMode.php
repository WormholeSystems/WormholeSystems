<?php

declare(strict_types=1);

namespace App\Enums;

/**
 * Whether a filter rule requires a match (include) or forbids one (exclude).
 */
enum KillmailFilterMode: string
{
    case Include = 'include';
    case Exclude = 'exclude';

    public function label(): string
    {
        return match ($this) {
            self::Include => 'Must match',
            self::Exclude => 'Must not match',
        };
    }
}
