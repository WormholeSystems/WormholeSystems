<?php

declare(strict_types=1);

namespace App\Enums;

enum ConnectionType: string
{
    case Wormhole = 'wormhole';
    case Stargate = 'stargate';
}
