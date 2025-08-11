<?php

declare(strict_types=1);

namespace App\Enums;

enum ShipSize: string
{
    case Frigate = 'frigate';
    case Medium = 'medium';
    case Large = 'large';
}
