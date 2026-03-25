<?php

declare(strict_types=1);

namespace App\Enums;

enum ThreatLevel: string
{
    case Critical = 'critical';
    case High = 'high';
    case Unknown = 'unknown';
}
