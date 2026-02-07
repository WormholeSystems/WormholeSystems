<?php

declare(strict_types=1);

use App\Console\Commands\Characters\GetOnlineCharacterLocationsCommand;
use App\Console\Commands\Characters\GetOnlineCharactersCommand;
use App\Console\Commands\CheckConnectionAgeCommand;
use App\Console\Commands\GenerateStaticDataCommand;
use App\Console\Commands\GetServerStatusCommand;
use App\Console\Commands\Killmails\GetKillmailsForLast90DaysCommand;
use App\Console\Commands\Killmails\PurgeOldKillmailsCommand;
use App\Console\Commands\Signatures\DeleteOldSignaturesCommand;
use App\Console\Commands\Sovereignty\GetSovereigntiesCommand;
use Illuminate\Queue\Console\PruneBatchesCommand;
use Illuminate\Support\Facades\Schedule;

Schedule::command(GetServerStatusCommand::class)->runInBackground()->everyMinute()->withoutOverlapping();
Schedule::command(GetOnlineCharactersCommand::class)->runInBackground()->everyThirtySeconds()->withoutOverlapping(expiresAt: 60)->notDuringDowntime();
Schedule::command(GetOnlineCharacterLocationsCommand::class)->runInBackground()->everyFiveSeconds()->withoutOverlapping(expiresAt: 60)->notDuringDowntime();
Schedule::command(GetSovereigntiesCommand::class)->runInBackground()->daily()->at('15:00')->withoutOverlapping()->notDuringDowntime();
Schedule::command(GenerateStaticDataCommand::class)->runInBackground()->daily()->withoutOverlapping()->notDuringDowntime();
Schedule::command(CheckConnectionAgeCommand::class)->runInBackground()->everyTenMinutes()->withoutOverlapping();
Schedule::command(DeleteOldSignaturesCommand::class)->runInBackground()->everyTenMinutes()->withoutOverlapping();
Schedule::command(GetKillmailsForLast90DaysCommand::class)->runInBackground()->weekly();
Schedule::command(PurgeOldKillmailsCommand::class)->runInBackground()->daily();
Schedule::command(PruneBatchesCommand::class)->daily();
