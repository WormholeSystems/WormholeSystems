<?php

namespace App\Console\Commands\Killmails;

use Carbon\CarbonImmutable;
use Illuminate\Console\Command;

use function Laravel\Prompts\progress;

class GetKillmailsForLast90DaysCommand extends Command
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
            $days,
            function (int $step) use ($date) {
                $this->callSilently('app:get-killmails-for-day', [
                    'date' => $date->addDays($step)->toDateString(),
                ]);
            });

        return self::SUCCESS;
    }
}
