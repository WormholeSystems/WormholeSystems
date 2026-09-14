<?php

declare(strict_types=1);

namespace App\Enums;

enum MapAlertType: string
{
    case Proximity = 'proximity';
    case Killmail = 'killmail';
    case JumpRange = 'jump_range';
    case MaintainerPodium = 'maintainer_podium';

    public function label(): string
    {
        return match ($this) {
            self::Proximity => 'System near chain',
            self::Killmail => 'Kills near chain',
            self::JumpRange => 'Capital jump range',
            self::MaintainerPodium => 'Monthly maintainer podium',
        };
    }
}
