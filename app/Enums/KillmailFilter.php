<?php

declare(strict_types=1);

namespace App\Enums;

enum KillmailFilter: string
{
    case All = 'all';
    case JSpace = 'jspace';
    case KSpace = 'kspace';

}
