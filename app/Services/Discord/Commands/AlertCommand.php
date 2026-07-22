<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

use App\Actions\Discord\CreateDiscordAlertAction;
use App\Enums\JumpShipType;
use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertMentionMode;
use App\Enums\MapAlertType;
use App\Models\DiscordAccount;
use Discord\Parts\Interactions\ApplicationCommand;

/**
 * The alert creation command: destination groups (dm, channel) each carry one
 * subcommand per trigger type, so every leaf exposes exactly the options that are
 * valid for that combination.
 */
final readonly class AlertCommand implements CommandDefinition
{
    use ReadsInteractionOptions;

    public function __construct(private CreateDiscordAlertAction $createAlert) {}

    public function definition(): SlashCommand
    {
        return SlashCommand::make('alert', 'Create a map alert')->options(
            SubCommandGroup::make('dm', 'Alerts delivered by direct message')->subCommands(...$this->variants(false)),
            SubCommandGroup::make('channel', 'Alerts posted in this channel')->subCommands(...$this->variants(true)),
        );
    }

    public function requiresLinkedAccount(): bool
    {
        return true;
    }

    public function respond(ApplicationCommand $interaction, ?DiscordAccount $account): string
    {
        $group = $this->subcommand($interaction);
        $variant = $group->options->first();

        $deliveryType = match ($group->name) {
            'dm' => MapAlertDeliveryType::DiscordDm,
            'channel' => MapAlertDeliveryType::DiscordChannel,
            default => null,
        };
        $type = match ($variant->name) {
            'proximity' => MapAlertType::Proximity,
            'jump-range' => MapAlertType::JumpRange,
            'killmail' => MapAlertType::Killmail,
            default => null,
        };
        if ($deliveryType === null || $type === null) {
            return 'That alert command is unavailable.';
        }

        $roleId = $this->option($variant, 'role');
        $mentionMode = $deliveryType === MapAlertDeliveryType::DiscordDm
            ? MapAlertMentionMode::None
            : MapAlertMentionMode::tryFrom((string) ($this->option($variant, 'mention') ?? ($roleId === null ? 'none' : 'role')));

        $error = $this->deniedChannelPermission($interaction, $deliveryType, $mentionMode);
        if ($error !== null) {
            return $error;
        }

        $systemId = $this->option($variant, 'system');
        $jumps = $this->option($variant, 'jumps');
        $jdcLevel = $this->option($variant, 'jdc');
        $from = $this->option($variant, 'from');

        return $this->createAlert->handle(
            $account,
            $type,
            $deliveryType,
            (int) $this->option($variant, 'map'),
            $systemId === null ? null : (int) $systemId,
            $jumps === null ? null : (int) $jumps,
            JumpShipType::tryFrom((string) $this->option($variant, 'ship')),
            $jdcLevel === null ? null : (int) $jdcLevel,
            (bool) $this->option($variant, 'highsec'),
            $mentionMode,
            $interaction->guild_id === null ? null : (string) $interaction->guild_id,
            $interaction->channel_id === null ? null : (string) $interaction->channel_id,
            $roleId === null ? null : (string) $roleId,
            $from === null ? null : (int) $from,
        );
    }

    /** @return SubCommand[] */
    private function variants(bool $withMentions): array
    {
        return collect([
            new ProximityAlertCommand,
            new JumpRangeAlertCommand,
            new KillmailAlertCommand,
        ])->map(fn (AlertVariantDefinition $variant): SubCommand => $variant->definition($withMentions))->all();
    }

    private function deniedChannelPermission(ApplicationCommand $interaction, MapAlertDeliveryType $deliveryType, ?MapAlertMentionMode $mentionMode): ?string
    {
        if ($deliveryType !== MapAlertDeliveryType::DiscordChannel) {
            return null;
        }

        $member = $interaction->member;
        $isAdministrator = $member !== null && $member->permissions->administrator;

        if (! $isAdministrator && ($member === null || ! $member->permissions->manage_channels)) {
            return 'You need the Manage Channels permission to create an alert in this channel.';
        }

        if ($mentionMode === MapAlertMentionMode::Role && (! $isAdministrator && ! $member->permissions->manage_roles)) {
            return 'You need the Manage Roles permission to configure a role mention.';
        }

        if ($mentionMode === MapAlertMentionMode::Everyone && (! $isAdministrator && ! $member->permissions->mention_everyone)) {
            return 'You need the Mention Everyone permission to configure an everyone mention.';
        }

        return null;
    }
}
