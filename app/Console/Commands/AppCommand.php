<?php

declare(strict_types=1);

namespace App\Console\Commands;

use Illuminate\Console\Command;
use Illuminate\Support\Facades\Log;

use function Laravel\Prompts\error;
use function Laravel\Prompts\info;
use function now;
use function sprintf;

abstract class AppCommand extends Command
{
    public function info($string, $verbosity = null, bool $log = false): void
    {
        info(sprintf('[%s] %s', now()->toDateTimeString(), $string));
        Log::when($log)->info(sprintf('[%s] %s', $this->signature, $string));
    }

    public function error($string, $verbosity = null, bool $log = false): void
    {
        error(sprintf('[%s] %s', now()->toDateTimeString(), $string));
        Log::when($log)->error(sprintf('[%s] %s', $this->signature, $string));
    }
}
