<?php

use App\Console\Commands\Characters\GetOnlineCharacterLocationsCommand;
use App\Console\Commands\Characters\GetOnlineCharactersCommand;
use App\Console\Commands\CheckConnectionAgeCommand;
use Illuminate\Support\Facades\Schedule;

Schedule::command(GetOnlineCharactersCommand::class)->everyMinute();
Schedule::command(GetOnlineCharacterLocationsCommand::class)->everyTenSeconds();
Schedule::command(CheckConnectionAgeCommand::class)->everyTenMinutes();
