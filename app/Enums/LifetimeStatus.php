<?php

declare(strict_types=1);

namespace App\Enums;

enum LifetimeStatus: string
{
    case Healthy = 'healthy';
    case EndOfLife = 'eol'; // <4 hours remaining
    case Critical = 'critical'; // <1 hour remaining
    case Expired = 'expired'; // reliable lifetime over, closure imminent

    public function severity(): int
    {
        return match ($this) {
            self::Healthy => 1,
            self::EndOfLife => 2,
            self::Critical => 3,
            self::Expired => 4,
        };
    }
}
