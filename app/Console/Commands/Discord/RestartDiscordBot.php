<?php

declare(strict_types=1);

namespace App\Console\Commands\Discord;

use Illuminate\Console\Attributes\Description;
use Illuminate\Console\Attributes\Signature;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Str;

#[Signature('discord:restart')]
#[Description('Signal the Discord bot process to restart gracefully.')]
final class RestartDiscordBot extends Command
{
    public function handle(): int
    {
        Cache::put('discord.restart', (string) Str::uuid());
        $this->components->info('Discord bot restart signal sent.');

        return self::SUCCESS;
    }
}
