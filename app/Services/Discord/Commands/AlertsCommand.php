<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

use App\Actions\Discord\DeleteDiscordAlertAction;
use App\Actions\Discord\DisableDiscordAlertAction;
use App\Actions\Discord\EnableDiscordAlertAction;
use App\Actions\Discord\ListDiscordMapAlertsAction;
use App\Actions\Discord\ListMyDiscordAlertsAction;
use App\Models\DiscordAccount;
use Discord\Parts\Interactions\ApplicationCommand;

final readonly class AlertsCommand implements CommandDefinition
{
    use BuildsCommandOptions;
    use ReadsInteractionOptions;

    public function __construct(
        private ListMyDiscordAlertsAction $listMyAlerts,
        private ListDiscordMapAlertsAction $listMapAlerts,
        private EnableDiscordAlertAction $enableAlert,
        private DisableDiscordAlertAction $disableAlert,
        private DeleteDiscordAlertAction $deleteAlert,
    ) {}

    public function definition(): SlashCommand
    {
        return SlashCommand::make('alerts', 'Manage your map alerts')->options(
            SubCommand::make('list', 'List alerts you created'),
            SubCommand::make('map', 'List alerts visible for a map')->options($this->mapOption()),
            SubCommand::make('enable', 'Enable one of your alerts')->options($this->alertOption()),
            SubCommand::make('disable', 'Disable an alert')->options($this->alertOption()),
            SubCommand::make('remove', 'Remove an alert')->options($this->alertOption()),
        );
    }

    public function requiresLinkedAccount(): bool
    {
        return true;
    }

    public function respond(ApplicationCommand $interaction, ?DiscordAccount $account): string
    {
        $subcommand = $this->subcommand($interaction);

        return match ($subcommand->name) {
            'list' => $this->listMyAlerts->handle($account),
            'map' => $this->listMapAlerts->handle($account, (int) $this->option($subcommand, 'map')),
            'enable' => $this->enableAlert->handle($account, (int) $this->option($subcommand, 'alert')),
            'disable' => $this->disableAlert->handle($account, (int) $this->option($subcommand, 'alert')),
            'remove' => $this->deleteAlert->handle($account, (int) $this->option($subcommand, 'alert')),
            default => 'That alert command is unavailable.',
        };
    }
}
