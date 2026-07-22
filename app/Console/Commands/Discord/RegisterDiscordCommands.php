<?php

declare(strict_types=1);

namespace App\Console\Commands\Discord;

use App\Services\Discord\Commands\CommandDefinition;
use App\Services\Discord\Commands\CommandRegistry;
use App\Services\Discord\DiscordHttp;
use Illuminate\Console\Attributes\Description;
use Illuminate\Console\Attributes\Signature;
use Illuminate\Console\Command;

#[Signature('discord:register-commands {--global : Register globally instead of using DISCORD_TEST_GUILD_ID}')]
#[Description('Register or update the Discord slash commands.')]
final class RegisterDiscordCommands extends Command
{
    public function handle(DiscordHttp $http, CommandRegistry $registry): int
    {
        $applicationId = config('services.discord.application_id');
        $token = config('services.discord.bot_token');
        if (! is_string($applicationId) || $applicationId === '' || ! is_string($token) || $token === '') {
            $this->components->error('Discord application ID and bot token are required.');

            return self::FAILURE;
        }

        $guildId = $this->option('global') ? null : config('services.discord.test_guild_id');
        if (! $this->option('global') && (! is_string($guildId) || $guildId === '')) {
            $this->components->error('DISCORD_TEST_GUILD_ID is required unless --global is used.');

            return self::FAILURE;
        }
        $endpoint = '/applications/'.$applicationId.($guildId ? '/guilds/'.$guildId : '').'/commands';
        $payload = collect($registry->all())
            ->map(fn (CommandDefinition $command): array => $command->definition()->toArray())
            ->all();
        $http->bot()->put($endpoint, $payload)->throw();
        $this->components->info($guildId ? 'Guild commands registered.' : 'Global commands registered.');

        return self::SUCCESS;
    }
}
