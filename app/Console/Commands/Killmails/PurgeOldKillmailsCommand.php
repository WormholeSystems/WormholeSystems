<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Console\Commands\AppCommand;
use App\Models\Killmail;
use Illuminate\Container\Attributes\Config;

final class PurgeOldKillmailsCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:purge-old-killmails';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Purge old killmails from the database';

    /**
     * Execute the console command.
     */
    public function handle(
        #[Config('services.zkillboard.max_age_days')] int $max_age_days,
    ): int {
        $deleted = Killmail::query()
            ->where('time', '<', now()->subDays($max_age_days))
            ->delete();

        $this->info("Deleted $deleted killmails older than $max_age_days days.");

        return 0;
    }
}
