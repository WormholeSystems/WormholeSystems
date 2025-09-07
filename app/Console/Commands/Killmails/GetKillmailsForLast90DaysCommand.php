<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Console\Commands\AppCommand;
use Carbon\CarbonImmutable;

use function Laravel\Prompts\progress;

final class GetKillmailsForLast90DaysCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:get-killmails-for-last-90-days';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Download and process killmails for the last 90 days';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $date = CarbonImmutable::now()->subDays(90);

        $days = CarbonImmutable::now()->diffInDays($date, true);
        progress('Downloading killmails for the last 90 days',
            (int) $days,
            function (int $step) use ($date): void {
                $this->callSilently('app:get-killmails-for-day', [
                    'date' => $date->addDays($step)->toDateString(),
                ]);
            });

        $this->info('Finished downloading and processing killmails for the last 90 days');

        return self::SUCCESS;
    }
}
