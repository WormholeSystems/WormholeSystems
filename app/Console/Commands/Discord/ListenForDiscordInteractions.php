<?php

declare(strict_types=1);

namespace App\Console\Commands\Discord;

use App\Services\Discord\DiscordInteractionHandler;
use Discord\Discord;
use Discord\WebSockets\Intents;
use Illuminate\Console\Attributes\Description;
use Illuminate\Console\Attributes\Signature;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Cache;
use Throwable;

#[Signature('discord:listen')]
#[Description('Run the long-lived Discord bot gateway process.')]
final class ListenForDiscordInteractions extends Command
{
    public function handle(DiscordInteractionHandler $handler): int
    {
        $token = config('services.discord.bot_token');
        if (! is_string($token) || $token === '') {
            $this->components->error('DISCORD_BOT_TOKEN is not configured.');

            return self::FAILURE;
        }

        $discord = new Discord(['token' => $token, 'intents' => Intents::GUILDS]);
        $this->trap([SIGTERM, SIGQUIT], fn () => $discord->close());
        $restartSignal = Cache::get('discord.restart');
        $discord->getLoop()->addPeriodicTimer(5, function () use ($discord, $restartSignal): void {
            try {
                if (Cache::get('discord.restart') !== $restartSignal) {
                    $discord->close();
                }
            } catch (Throwable $exception) {
                report($exception);
            }
        });
        $discord->on('init', function (Discord $discord) use ($handler): void {
            $handler->listen($discord);
            $this->components->info('Discord bot is ready.');
        });
        $discord->run();

        return self::SUCCESS;
    }
}
