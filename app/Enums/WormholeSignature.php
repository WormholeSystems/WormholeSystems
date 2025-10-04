<?php

declare(strict_types=1);

namespace App\Enums;

enum WormholeSignature: string
{
    // K162 - Universal exit wormhole
    case K162 = 'K162';

    // Universal spawns (can appear from any system)
    case A009 = 'A009';
    case B735 = 'B735';
    case C008 = 'C008';
    case C414 = 'C414';
    case E004 = 'E004';
    case G008 = 'G008';
    case J492 = 'J492';
    case L005 = 'L005';
    case M001 = 'M001';
    case Q003 = 'Q003';
    case R259 = 'R259';
    case S877 = 'S877';
    case V928 = 'V928';
    case Z006 = 'Z006';

    // C1 specific
    case H121 = 'H121';
    case C125 = 'C125';
    case O883 = 'O883';
    case M609 = 'M609';
    case L614 = 'L614';
    case S804 = 'S804';
    case N110 = 'N110';
    case J244 = 'J244';
    case Z060 = 'Z060';
    case F353 = 'F353';

    // C2 specific
    case Z647 = 'Z647';
    case D382 = 'D382';
    case O477 = 'O477';
    case Y683 = 'Y683';
    case N062 = 'N062';
    case R474 = 'R474';
    case B274 = 'B274';
    case A239 = 'A239';
    case E545 = 'E545';
    case F135 = 'F135';

    // C3 specific
    case V301 = 'V301';
    case I182 = 'I182';
    case N968 = 'N968';
    case T405 = 'T405';
    case N770 = 'N770';
    case A982 = 'A982';
    case D845 = 'D845';
    case U210 = 'U210';
    case K346 = 'K346';

    // C4 specific
    case P060 = 'P060';
    case N766 = 'N766';
    case C247 = 'C247';
    case X877 = 'X877';
    case H900 = 'H900';
    case U574 = 'U574';
    case S047 = 'S047';
    case N290 = 'N290';
    case K329 = 'K329';

    // C5 specific
    case Y790 = 'Y790';
    case D364 = 'D364';
    case M267 = 'M267';
    case E175 = 'E175';
    case H296 = 'H296';
    case V753 = 'V753';
    case D792 = 'D792';
    case C140 = 'C140';
    case Z142 = 'Z142';

    // C6 specific
    case Q317 = 'Q317';
    case G024 = 'G024';
    case L477 = 'L477';
    case Z457 = 'Z457';
    case V911 = 'V911';
    case W237 = 'W237';
    case B520 = 'B520';
    case C391 = 'C391';
    case C248 = 'C248';

    // Highsec specific
    case Z971 = 'Z971';
    case R943 = 'R943';
    case X702 = 'X702';
    case O128 = 'O128';
    case M555 = 'M555';
    case B041 = 'B041';
    case A641 = 'A641';
    case R051 = 'R051';
    case V283 = 'V283';
    case T458 = 'T458';

    // Lowsec/Nullsec specific
    case N432 = 'N432';
    case U319 = 'U319';
    case B449 = 'B449';
    case N944 = 'N944';
    case S199 = 'S199';
    case M164 = 'M164';
    case L031 = 'L031';

    // Pochven specific
    case R081 = 'R081';
    case X450 = 'X450';
    case C729 = 'C729';
    case U372 = 'U372';
    case F216 = 'F216';

    // Thera specific
    case Q063 = 'Q063';
    case V898 = 'V898';
    case E587 = 'E587';

    // Special entries
    case J377 = 'J377'; // Turnur
    case U201 = 'U201';
}
