<?php

declare(strict_types=1);

namespace App\Enums;

enum Permission: string
{
    case Guest = 'guest';
    case Read = 'read';
    case Write = 'write';
}
