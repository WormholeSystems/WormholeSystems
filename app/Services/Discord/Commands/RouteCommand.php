<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

use App\Actions\Discord\CalculateDiscordRouteAction;
use App\Models\DiscordAccount;
use Discord\Parts\Interactions\ApplicationCommand;

final readonly class RouteCommand implements CommandDefinition
{
    use BuildsCommandOptions;
    use ReadsInteractionOptions;

    public function __construct(private CalculateDiscordRouteAction $calculateRoute) {}

    public function definition(): SlashCommand
    {
        return SlashCommand::make('route', 'Find the shortest route from a map')->options(
            $this->mapOption(),
            $this->systemOption(),
            $this->fromOption(),
        );
    }

    public function requiresLinkedAccount(): bool
    {
        return true;
    }

    public function respond(ApplicationCommand $interaction, ?DiscordAccount $account): string
    {
        $from = $this->option($interaction->data, 'from');

        return $this->calculateRoute->handle(
            $account,
            (int) $this->option($interaction->data, 'map'),
            (int) $this->option($interaction->data, 'system'),
            $from === null ? null : (int) $from,
        );
    }
}
