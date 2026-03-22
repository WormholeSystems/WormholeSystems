<?php

declare(strict_types=1);

namespace App\Enums;

enum zKillboardCacheKey: string
{
    case Restart = 'zkillboard:r2z2:restart';
    case LastSequence = 'zkillboard:r2z2:last_sequence';
}
