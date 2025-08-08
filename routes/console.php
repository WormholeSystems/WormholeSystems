<?php

use App\Console\Commands\Characters\GetOnlineCharacterLocationsCommand;
use App\Console\Commands\Characters\GetOnlineCharactersCommand;
use App\Console\Commands\CheckConnectionAgeCommand;
use App\Console\Commands\GetServerStatusCommand;
use App\Console\Commands\Signatures\DeleteOldSignaturesCommand;
use App\Console\Commands\Sovereignty\GetSovereigntiesCommand;
use Illuminate\Support\Facades\Schedule;

Schedule::command(GetOnlineCharactersCommand::class)->everyMinute()->withoutOverlapping();
Schedule::command(GetOnlineCharacterLocationsCommand::class)->everyTenSeconds()->withoutOverlapping();
Schedule::command(CheckConnectionAgeCommand::class)->everyTenMinutes()->withoutOverlapping();
Schedule::command(GetSovereigntiesCommand::class)->daily()->at('15:00')->withoutOverlapping();
Schedule::command(DeleteOldSignaturesCommand::class)->everyTenMinutes()->withoutOverlapping();
Schedule::command(GetServerStatusCommand::class)->everyMinute()->withoutOverlapping();
