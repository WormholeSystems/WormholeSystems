<?php

declare(strict_types=1);

namespace App\Console\Commands\Sovereignty;

use App\Jobs\Sovereignty\GetSovereignties;
use Illuminate\Console\Command;

final class GetSovereigntiesCommand extends Command
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
