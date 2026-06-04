<?php

declare(strict_types=1);

namespace App\Enums;

/**
 * The attribute of a killmail a single filter rule inspects.
 */
enum KillmailFilterSubject: string
{
    case ShipType = 'ship_type';
    case ShipGroup = 'ship_group';
    case Character = 'character';
    case Corporation = 'corporation';
    case Alliance = 'alliance';

    public function label(): string
    {
        return match ($this) {
            self::ShipType => 'Type',
            self::ShipGroup => 'Group',
            self::Character => 'Character',
            self::Corporation => 'Corporation',
            self::Alliance => 'Alliance',
        };
    }
}
