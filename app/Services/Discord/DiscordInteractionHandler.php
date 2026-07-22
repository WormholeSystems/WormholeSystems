<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Actions\Discord\AutocompleteDiscordAlertsAction;
use App\Actions\Discord\CalculateDiscordRouteAction;
use App\Actions\Discord\CreateDiscordAccountLinkAction;
use App\Actions\Discord\CreateDiscordProximityAlertAction;
use App\Actions\Discord\DeleteDiscordAlertAction;
use App\Actions\Discord\DisableDiscordAlertAction;
use App\Actions\Discord\EnableDiscordAlertAction;
use App\Actions\Discord\GetDiscordAccountStatusAction;
use App\Actions\Discord\ListDiscordMapAlertsAction;
use App\Actions\Discord\ListMyDiscordAlertsAction;
use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertMentionMode;
use App\Models\DiscordAccount;
use Closure;
use Discord\Builders\MessageBuilder;
use Discord\Discord;
use Discord\Helpers\Collection;
use Discord\Parts\Interactions\ApplicationCommand;
use Discord\Parts\Interactions\ApplicationCommandAutocomplete;
use Discord\Parts\Interactions\Command\Choice;
use Discord\Parts\Interactions\Request\InteractionData;
use Discord\Parts\Interactions\Request\Option;
use Discord\WebSockets\Event;
use Illuminate\Support\Facades\Context;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;
use Throwable;

final readonly class DiscordInteractionHandler
{
    public function __construct(
        private DiscordAutocomplete $autocomplete,
        private GetDiscordAccountStatusAction $getAccountStatus,
        private CreateDiscordAccountLinkAction $createAccountLink,
        private ListMyDiscordAlertsAction $listMyAlerts,
        private ListDiscordMapAlertsAction $listMapAlerts,
        private CreateDiscordProximityAlertAction $createProximityAlert,
        private EnableDiscordAlertAction $enableAlert,
        private DisableDiscordAlertAction $disableAlert,
        private DeleteDiscordAlertAction $deleteAlert,
        private AutocompleteDiscordAlertsAction $autocompleteAlerts,
        private CalculateDiscordRouteAction $calculateRoute,
    ) {}

    public function listen(Discord $discord): void
    {
        $handleAlerts = fn (ApplicationCommand $interaction, Collection $params) => $this->execute($interaction, fn () => $this->alerts($interaction));

        $discord->listenCommand('account', fn (ApplicationCommand $interaction, Collection $params) => $this->execute($interaction, fn () => $this->account($interaction)));
        $discord->listenCommand('alerts', $handleAlerts)->addSubCommand('add', $handleAlerts);
        $discord->listenCommand('route', fn (ApplicationCommand $interaction, Collection $params) => $this->execute($interaction, fn () => $this->route($interaction)));
        $discord->on(Event::INTERACTION_CREATE, function ($interaction) use ($discord): void {
            if ($interaction instanceof ApplicationCommandAutocomplete && in_array($interaction->data->name, ['alerts', 'route'], true)) {
                $this->safeAutocomplete($discord, $interaction);
            }
        });
    }

    private function execute(ApplicationCommand $interaction, Closure $callback): void
    {
        $interaction->acknowledgeWithResponse(true)->then(
            fn () => Context::scope(function () use ($interaction, $callback): void {
                try {
                    $callback();
                } catch (Throwable $exception) {
                    report($exception);
                    $this->respond($interaction, 'Something went wrong while handling that command.');
                }
            }),
            fn (Throwable $exception) => report($exception),
        );
    }

    private function safeAutocomplete(Discord $discord, ApplicationCommandAutocomplete $interaction): void
    {
        Context::scope(function () use ($discord, $interaction): void {
            $focused = $this->focusedOption($interaction);
            Log::debug('Discord autocomplete requested.', [
                'command' => $interaction->data->name,
                'option' => $focused?->name,
                'value' => $focused?->value,
                'discord_user_id' => $interaction->user?->id,
            ]);

            try {
                $choices = array_map(
                    fn (array $choice): Choice => Choice::new($discord, $choice['name'], $choice['value']),
                    $this->autocomplete($interaction),
                );
            } catch (Throwable $exception) {
                report($exception);
                $choices = [];
            }

            Log::debug('Discord autocomplete completed.', ['choices' => count($choices)]);

            $interaction->autoCompleteResult($choices)->then(null, fn (Throwable $exception) => report($exception));
        });
    }

    private function account(ApplicationCommand $interaction): void
    {
        $subcommand = $this->subcommand($interaction);
        $account = $this->accountFor($interaction);

        $this->respond($interaction, match ($subcommand->name) {
            'status' => $this->getAccountStatus->handle($account),
            'link' => $this->createAccountLink->handle($account, $this->identity($interaction)),
            default => 'That account command is unavailable.',
        });
    }

    private function alerts(ApplicationCommand $interaction): void
    {
        $account = $this->accountFor($interaction);
        if (! $account instanceof DiscordAccount) {
            $this->respond($interaction, 'Link your account first with `/account link`.');

            return;
        }

        $subcommand = $this->subcommand($interaction);
        $content = match ($subcommand->name) {
            'list' => $this->listMyAlerts->handle($account),
            'map' => $this->listMapAlerts->handle($account, (int) $this->option($subcommand, 'map')),
            'enable' => $this->enableAlert->handle($account, (int) $this->option($subcommand, 'alert')),
            'disable' => $this->disableAlert->handle($account, (int) $this->option($subcommand, 'alert')),
            'remove' => $this->deleteAlert->handle($account, (int) $this->option($subcommand, 'alert')),
            'add' => $this->createAlert($interaction, $account, $subcommand),
            default => 'That alert command is unavailable.',
        };

        $this->respond($interaction, $content);
    }

    private function createAlert(ApplicationCommand $interaction, DiscordAccount $account, Option $subcommand): string
    {
        $deliveryType = match ((string) $this->option($subcommand, 'destination')) {
            'dm' => MapAlertDeliveryType::DiscordDm,
            'channel' => MapAlertDeliveryType::DiscordChannel,
            default => null,
        };
        $mentionMode = MapAlertMentionMode::tryFrom((string) ($this->option($subcommand, 'mention') ?? 'none'));
        $canManageChannels = $interaction->member !== null
            && ($interaction->member->permissions->administrator || $interaction->member->permissions->manage_channels);
        $canManageRoles = $interaction->member !== null
            && ($interaction->member->permissions->administrator || $interaction->member->permissions->manage_roles);

        if ($deliveryType === MapAlertDeliveryType::DiscordChannel
            && ! $canManageChannels) {
            return 'You need the Manage Channels permission to create an alert in this channel.';
        }

        if ($deliveryType === MapAlertDeliveryType::DiscordChannel
            && $mentionMode === MapAlertMentionMode::Role
            && ! $canManageRoles) {
            return 'You need the Manage Roles permission to configure a role mention.';
        }

        $canMentionEveryone = $interaction->member !== null
            && ($interaction->member->permissions->administrator || $interaction->member->permissions->mention_everyone);

        if ($deliveryType === MapAlertDeliveryType::DiscordChannel
            && $mentionMode === MapAlertMentionMode::Everyone
            && ! $canMentionEveryone) {
            return 'You need the Mention Everyone permission to configure an everyone mention.';
        }

        return $this->createProximityAlert->handle(
            $account,
            (int) $this->option($subcommand, 'map'),
            (int) $this->option($subcommand, 'system'),
            (int) $this->option($subcommand, 'jumps'),
            $deliveryType,
            $mentionMode,
            $interaction->guild_id === null ? null : (string) $interaction->guild_id,
            $interaction->channel_id === null ? null : (string) $interaction->channel_id,
            ($roleId = $this->option($subcommand, 'role')) === null ? null : (string) $roleId,
        );
    }

    private function route(ApplicationCommand $interaction): void
    {
        $account = $this->accountFor($interaction);
        if (! $account instanceof DiscordAccount) {
            $this->respond($interaction, 'Link your account first with `/account link`.');

            return;
        }

        $this->respond($interaction, $this->calculateRoute->handle(
            $account,
            (int) $this->option($interaction->data, 'map'),
            (int) $this->option($interaction->data, 'system'),
        ));
    }

    /** @return array<int, array{name: string, value: string}> */
    private function autocomplete(ApplicationCommandAutocomplete $interaction): array
    {
        $focused = $this->focusedOption($interaction);
        if (! $focused instanceof Option) {
            return [];
        }

        $account = $this->accountFor($interaction);
        if (! $account instanceof DiscordAccount) {
            return [];
        }

        return match ($focused->name) {
            'map' => $this->autocomplete->maps($account, (string) $focused->value),
            'system' => $this->autocomplete->solarsystems((string) $focused->value),
            'alert' => $this->autocompleteAlerts->handle($account, (string) $focused->value),
            default => [],
        };
    }

    private function accountFor(ApplicationCommand|ApplicationCommandAutocomplete $interaction): ?DiscordAccount
    {
        return DiscordAccount::query()->where('discord_user_id', (string) $interaction->user->id)->with(['user.characters'])->first();
    }

    private function subcommand(ApplicationCommand $interaction): Option
    {
        return $interaction->data->options->first();
    }

    private function option(Option|InteractionData $source, string $name): mixed
    {
        return $source->options->get('name', $name)?->value;
    }

    private function focusedOption(ApplicationCommandAutocomplete $interaction): ?Option
    {
        foreach ($interaction->data->options as $option) {
            if ($option->focused) {
                return $option;
            }
            foreach ($option->options ?? [] as $nested) {
                if ($nested->focused) {
                    return $nested;
                }
            }
        }

        return null;
    }

    /** @return array{id: string, username: string, global_name: string|null, avatar: string|null} */
    private function identity(ApplicationCommand $interaction): array
    {
        $attributes = $interaction->user->getRawAttributes();

        return [
            'id' => (string) $interaction->user->id,
            'username' => $interaction->user->username,
            'global_name' => $attributes['global_name'] ?? null,
            'avatar' => $attributes['avatar'] ?? null,
        ];
    }

    private function respond(ApplicationCommand $interaction, string $content): void
    {
        $builder = MessageBuilder::new()->setContent(Str::limit($content, 1900));
        $promise = $interaction->isResponded()
            ? $interaction->updateOriginalResponse($builder)
            : $interaction->respondWithMessage($builder, true);

        $promise->then(null, fn (Throwable $exception) => report($exception));
    }
}
