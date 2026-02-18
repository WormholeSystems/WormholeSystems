<?php

declare(strict_types=1);

namespace App\Enums;

enum RemovableCard: string
{
    case Audits = 'audits';
    case ShipHistory = 'ship-history';
    case Characters = 'characters';
    case Killmails = 'killmails';
    case Autopilot = 'autopilot';
    case EveScout = 'eve-scout';
}
