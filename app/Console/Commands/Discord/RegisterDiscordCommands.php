<?php

declare(strict_types=1);

namespace App\Console\Commands\Discord;

use App\Services\Discord\DiscordHttp;
use Illuminate\Console\Attributes\Description;
use Illuminate\Console\Attributes\Signature;
use Illuminate\Console\Command;

#[Signature('discord:register-commands {--global : Register globally instead of using DISCORD_TEST_GUILD_ID}')]
#[Description('Register or update the Discord slash commands.')]
final class RegisterDiscordCommands extends Command
{
    public function handle(DiscordHttp $http): int
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
        $http->bot()->put($endpoint, $this->commands())->throw();
        $this->components->info($guildId ? 'Guild commands registered.' : 'Global commands registered.');

        return self::SUCCESS;
    }

    /** @return array<int, array<string, mixed>> */
    private function commands(): array
    {
        $map = ['type' => 3, 'name' => 'map', 'description' => 'Wormhole Systems map', 'required' => true, 'autocomplete' => true];
        $system = ['type' => 3, 'name' => 'system', 'description' => 'EVE solar system', 'required' => true, 'autocomplete' => true];

        return [
            ['type' => 1, 'name' => 'account', 'description' => 'Manage your linked Wormhole Systems account', 'options' => [
                ['type' => 1, 'name' => 'link', 'description' => 'Link this Discord account'],
                ['type' => 1, 'name' => 'status', 'description' => 'Show your account link status'],
            ]],
            ['type' => 1, 'name' => 'alerts', 'description' => 'Manage personal proximity alerts', 'options' => [
                ['type' => 2, 'name' => 'add', 'description' => 'Create a proximity alert', 'options' => [
                    ['type' => 1, 'name' => 'dm', 'description' => 'Create a proximity alert delivered by direct message', 'options' => [
                        $map,
                        $system,
                        ['type' => 4, 'name' => 'jumps', 'description' => 'Maximum gate jumps', 'required' => true, 'min_value' => 1, 'max_value' => 20],
                    ]],
                    ['type' => 1, 'name' => 'channel', 'description' => 'Create a proximity alert posted in this channel', 'options' => [
                        $map,
                        $system,
                        ['type' => 4, 'name' => 'jumps', 'description' => 'Maximum gate jumps', 'required' => true, 'min_value' => 1, 'max_value' => 20],
                        ['type' => 3, 'name' => 'mention', 'description' => 'Who to ping', 'required' => true, 'choices' => [
                            ['name' => 'Nobody', 'value' => 'none'],
                            ['name' => 'Me', 'value' => 'creator'],
                            ['name' => 'A role', 'value' => 'role'],
                            ['name' => 'Everyone', 'value' => 'everyone'],
                        ]],
                        ['type' => 8, 'name' => 'role', 'description' => 'Role to ping', 'required' => false],
                    ]],
                ]],
                ['type' => 1, 'name' => 'list', 'description' => 'List alerts you created'],
                ['type' => 1, 'name' => 'map', 'description' => 'List alerts visible for a map', 'options' => [$map]],
                ['type' => 1, 'name' => 'enable', 'description' => 'Enable one of your alerts', 'options' => [
                    ['type' => 3, 'name' => 'alert', 'description' => 'Alert', 'required' => true, 'autocomplete' => true],
                ]],
                ['type' => 1, 'name' => 'disable', 'description' => 'Disable an alert', 'options' => [
                    ['type' => 3, 'name' => 'alert', 'description' => 'Alert', 'required' => true, 'autocomplete' => true],
                ]],
                ['type' => 1, 'name' => 'remove', 'description' => 'Remove a proximity alert', 'options' => [
                    ['type' => 3, 'name' => 'alert', 'description' => 'Alert', 'required' => true, 'autocomplete' => true],
                ]],
            ]],
            ['type' => 1, 'name' => 'route', 'description' => 'Find the shortest route from a map', 'options' => [$map, $system]],
        ];
    }
}
