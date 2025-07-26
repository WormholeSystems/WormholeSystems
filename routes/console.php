<?php

use App\Console\Commands\Characters\GetOnlineCharacterLocationsCommand;
use App\Console\Commands\Characters\GetOnlineCharactersCommand;
use App\Console\Commands\CheckConnectionAgeCommand;
use App\Console\Commands\Sovereignty\GetSovereigntiesCommand;
use Illuminate\Support\Facades\Schedule;

Schedule::command(GetOnlineCharactersCommand::class)->everyMinute()->withoutOverlapping();
Schedule::command(GetOnlineCharacterLocationsCommand::class)->everyTenSeconds()->withoutOverlapping();
Schedule::command(CheckConnectionAgeCommand::class)->everyTenMinutes()->withoutOverlapping();
Schedule::command(GetSovereigntiesCommand::class)->daily()->at('15:00')->withoutOverlapping();
