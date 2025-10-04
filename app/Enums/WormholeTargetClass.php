<?php

declare(strict_types=1);

namespace App\Enums;

enum WormholeTargetClass: string
{
    case C1 = 'C1';
    case C2 = 'C2';
    case C3 = 'C3';
    case C4 = 'C4';
    case C5 = 'C5';
    case C6 = 'C6';
    case C12 = 'C12'; // Thera
    case C13 = 'C13'; // Shattered
    case C14 = 'C14'; // Sentinel
    case C15 = 'C15'; // Barbican
    case C16 = 'C16'; // Vidette
    case C17 = 'C17'; // Conflux
    case C18 = 'C18'; // Redoubt
    case H = 'H';     // Highsec
    case L = 'L';     // Lowsec
    case N = 'N';     // Nullsec
    case Unknown = 'Unknown';
}


