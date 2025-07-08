<?php

namespace App\Enums;

enum MapSolarsystemStatus: string
{
    case Active = 'active';
    case Unknown = 'unknown';
    case Unscanned = 'unscanned';
    case Hostile = 'hostile';
    case Friendly = 'friendly';
    case Empty = 'empty';
}
