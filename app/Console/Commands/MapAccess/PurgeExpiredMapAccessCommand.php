<?php

declare(strict_types=1);

namespace App\Console\Commands\MapAccess;

use App\Console\Commands\AppCommand;
use App\Models\MapAccess;

final class PurgeExpiredMapAccessCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:purge-expired-map-access';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Purge expired map access entries';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $deleted = MapAccess::query()
            ->whereNotNull('expires_at')
            ->where('expires_at', '<=', now())
            ->delete();

        $this->info("Deleted $deleted expired map access entries.");

        return 0;
    }
}
