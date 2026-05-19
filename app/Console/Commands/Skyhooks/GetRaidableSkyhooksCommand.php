<?php

declare(strict_types=1);

namespace App\Console\Commands\Skyhooks;

use App\Console\Commands\AppCommand;
use App\Jobs\Skyhooks\GetRaidableSkyhooks;

final class GetRaidableSkyhooksCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:get-raidable-skyhooks-command {--sync}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get currently or shortly raidable skyhooks';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $sync = $this->option('sync');

        if ($sync) {
            return GetRaidableSkyhooks::dispatchSync();
        }

        GetRaidableSkyhooks::dispatch();

        return self::SUCCESS;
    }
}
