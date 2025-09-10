<?php

declare(strict_types=1);

namespace App\Enums;

enum LifetimeStatus: string
{
    case Healthy = 'healthy';
    case EndOfLife = 'eol'; // <4 hours remaining
    case Critical = 'critical'; // <1 hour remaining
}
