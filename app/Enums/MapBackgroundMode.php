<?php

declare(strict_types=1);

namespace App\Enums;

enum MapBackgroundMode: string
{
    case Grid = 'grid';
    case Viewport = 'viewport';
}
