<?php

declare(strict_types=1);

namespace App\Enums;

enum AliasScheme: string
{
    case Numeric = 'numeric';
    case Alphabetical = 'alphabetical';

    public const self DEFAULT = self::Numeric;

    /**
     * @return list<string>
     */
    public static function values(): array
    {
        return array_map(static fn (self $scheme): string => $scheme->value, self::cases());
    }
}
