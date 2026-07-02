<?php

declare(strict_types=1);

namespace App\Console\Commands\MapConnections;

use App\Console\Commands\AppCommand;
use App\Models\MapConnectionJump;

final class PruneUnclaimedConnectionJumpsCommand extends AppCommand
{
    public const int UNCLAIMED_LIFETIME_MINUTES = 10;

    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:prune-unclaimed-connection-jumps';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Deletes pending connection jumps that were never claimed by a connection';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $deleted = MapConnectionJump::query()
            ->whereNull('map_connection_id')
            ->where('created_at', '<', now()->subMinutes(self::UNCLAIMED_LIFETIME_MINUTES))
            ->delete();

        $this->info(sprintf('Deleted %d unclaimed connection jumps.', $deleted));

        return self::SUCCESS;
    }
}
