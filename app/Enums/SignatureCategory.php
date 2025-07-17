<?php

namespace App\Enums;

enum SignatureCategory: string
{
    case Wormhole = 'Wormhole';
    case DataSite = 'Data Site';
    case RelicSite = 'Relic Site';
    case CombatSite = 'Combat Site';
    case GasSite = 'Gas Site';
    case OreSite = 'Ore Site';
}
