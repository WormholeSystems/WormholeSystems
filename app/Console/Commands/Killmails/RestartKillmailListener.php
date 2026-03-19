<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use Illuminate\Console\Command;
use Illuminate\Support\Facades\Cache;

final class RestartKillmailListener extends Command
{
    protected $signature = 'app:restart-killmail-listener';

    protected $description = 'Signal the killmail listener to restart gracefully.';

    public function handle(): void
    {
        Cache::put(ListenForKillmails::RESTART_CACHE_KEY, now()->timestamp);

        $this->components->info('Killmail listener restart signal sent.');
    }
}
