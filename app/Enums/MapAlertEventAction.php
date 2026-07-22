<?php

declare(strict_types=1);

namespace App\Enums;

enum MapAlertEventAction: string
{
    case Created = 'created';
    case Updated = 'updated';
    case Enabled = 'enabled';
    case Disabled = 'disabled';
    case Removed = 'removed';
}
