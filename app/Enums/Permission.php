<?php

declare(strict_types=1);

namespace App\Enums;

enum Permission: string
{
    case Viewer = 'viewer';
    case Member = 'member';
    case Manager = 'manager';

    public function level(): int
    {
        return match ($this) {
            self::Viewer => 1,
            self::Member => 2,
            self::Manager => 3,
        };
    }

    public function isAtLeast(self $required): bool
    {
        return $this->level() >= $required->level();
    }
}
