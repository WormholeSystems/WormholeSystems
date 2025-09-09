<?php

declare(strict_types=1);

use App\Console\Commands\Characters\GetOnlineCharacterLocationsCommand;
use App\Console\Commands\Characters\GetOnlineCharactersCommand;
use App\Console\Commands\CheckConnectionAgeCommand;
use App\Console\Commands\GetServerStatusCommand;
use App\Console\Commands\Killmails\GetKillmailsForLast90DaysCommand;
use App\Console\Commands\Signatures\DeleteOldSignaturesCommand;
use App\Console\Commands\Sovereignty\GetSovereigntiesCommand;
use Illuminate\Support\Facades\Schedule;

Schedule::command(GetServerStatusCommand::class)->everyMinute()->withoutOverlapping();
Schedule::command(GetOnlineCharactersCommand::class)->everyMinute()->withoutOverlapping()->notDuringDowntime();
Schedule::command(GetOnlineCharacterLocationsCommand::class)->everyTenSeconds()->withoutOverlapping()->notDuringDowntime();
Schedule::command(GetSovereigntiesCommand::class)->daily()->at('15:00')->withoutOverlapping()->notDuringDowntime();
Schedule::command(CheckConnectionAgeCommand::class)->everyTenMinutes()->withoutOverlapping();
Schedule::command(DeleteOldSignaturesCommand::class)->everyTenMinutes()->withoutOverlapping();
Schedule::command(GetKillmailsForLast90DaysCommand::class)->weekly();
