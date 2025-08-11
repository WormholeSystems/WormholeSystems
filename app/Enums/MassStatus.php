<?php

declare(strict_types=1);

namespace App\Enums;

enum MassStatus: string
{
    case Fresh = 'fresh';
    case Reduced = 'reduced';
    case Critical = 'critical';
    case Unknown = 'unknown';
}
