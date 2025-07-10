<?php

namespace App\Enums;

enum Permission: string
{
    case Read = 'read';
    case Write = 'write';
}
