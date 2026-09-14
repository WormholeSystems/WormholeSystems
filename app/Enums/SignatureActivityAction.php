<?php

declare(strict_types=1);

namespace App\Enums;

enum SignatureActivityAction: string
{
    case Created = 'created';
    case Updated = 'updated';
    case Deleted = 'deleted';
}
