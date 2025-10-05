<?php

declare(strict_types=1);

namespace App\Enums;

enum RoutePreference: string
{
    case Shorter = 'shorter';
    case Safer = 'safer';
    case LessSecure = 'less_secure';
}
