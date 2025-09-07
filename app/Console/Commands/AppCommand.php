<?php

declare(strict_types=1);

namespace App\Console\Commands;

use Illuminate\Console\Command;

use function Laravel\Prompts\info;
use function now;
use function sprintf;

abstract class AppCommand extends Command
{
    public function info($string, $verbosity = null): void
    {
        info(sprintf('[%s] %s', now()->toDateTimeString(), $string));
    }
}
