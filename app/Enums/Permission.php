<?php

declare(strict_types=1);

namespace App\Enums;

enum Permission: string
{
    case Read = 'read';
    case Write = 'write';
}
