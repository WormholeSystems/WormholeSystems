<?php

namespace App\Enums;

enum MassStatus: string
{
    case Fresh = 'fresh';
    case Reduced = 'reduced';
    case Critical = 'critical';
    case Unknown = 'unknown';
}
