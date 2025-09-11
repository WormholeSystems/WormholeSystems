<?php

declare(strict_types=1);

namespace App\Console\Commands\Sovereignty;

use App\Console\Commands\AppCommand;
use App\Jobs\Sovereignty\GetSovereignties;

final class GetSovereigntiesCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:get-sovereignties-command {--sync}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get sovereignty of all systems';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $sync = $this->option('sync');

        if ($sync) {
            return GetSovereignties::dispatchSync();
        }

        GetSovereignties::dispatch();

        return self::SUCCESS;
    }
}
