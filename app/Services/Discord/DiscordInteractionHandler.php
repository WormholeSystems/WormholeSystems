<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Actions\Discord\AutocompleteDiscordAlertsAction;
use App\Models\DiscordAccount;
use App\Services\Discord\Commands\CommandDefinition;
use App\Services\Discord\Commands\CommandRegistry;
use Closure;
use Discord\Builders\MessageBuilder;
use Discord\Discord;
use Discord\Helpers\Collection;
use Discord\Parts\Interactions\ApplicationCommand;
use Discord\Parts\Interactions\ApplicationCommandAutocomplete;
use Discord\Parts\Interactions\Command\Choice;
use Discord\Parts\Interactions\Request\Option;
use Discord\WebSockets\Event;
use Illuminate\Support\Facades\Context;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;
use Throwable;

/**
 * Wires every registered slash command to the gateway: acknowledges interactions,
 * resolves the linked account, and lets each command definition build its own reply.
 */
final readonly class DiscordInteractionHandler
{
    public function __construct(
        private CommandRegistry $commands,
        private DiscordAutocomplete $autocomplete,
        private AutocompleteDiscordAlertsAction $autocompleteAlerts,
    ) {}

    public function listen(Discord $discord): void
    {
        foreach ($this->commands->all() as $command) {
            $discord->listenCommand(
                $command->definition()->name(),
                fn (ApplicationCommand $interaction, Collection $params) => $this->execute($interaction, fn () => $this->dispatch($command, $interaction)),
            );
        }

        $discord->on(Event::INTERACTION_CREATE, function ($interaction) use ($discord): void {
            if ($interaction instanceof ApplicationCommandAutocomplete && in_array($interaction->data->name, ['alert', 'alerts', 'route'], true)) {
                $this->safeAutocomplete($discord, $interaction);
            }
        });
    }

    private function dispatch(CommandDefinition $command, ApplicationCommand $interaction): void
    {
        $account = $this->accountFor($interaction);

        if ($command->requiresLinkedAccount() && ! $account instanceof DiscordAccount) {
            $this->respond($interaction, 'Link your account first with `/account link`.');

            return;
        }

        $this->respond($interaction, $command->respond($interaction, $account));
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
                    $this->autocompleteChoices($interaction),
                );
            } catch (Throwable $exception) {
                report($exception);
                $choices = [];
            }

            Log::debug('Discord autocomplete completed.', ['choices' => count($choices)]);

            $interaction->autoCompleteResult($choices)->then(null, fn (Throwable $exception) => report($exception));
        });
    }

    /** @return array<int, array{name: string, value: string}> */
    private function autocompleteChoices(ApplicationCommandAutocomplete $interaction): array
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

    private function focusedOption(ApplicationCommandAutocomplete $interaction): ?Option
    {
        return $this->findFocusedOption($interaction->data->options);
    }

    /**
     * @param  iterable<Option>  $options
     */
    private function findFocusedOption(iterable $options): ?Option
    {
        foreach ($options as $option) {
            if ($option->focused) {
                return $option;
            }

            $nested = $this->findFocusedOption($option->options ?? []);
            if ($nested instanceof Option) {
                return $nested;
            }
        }

        return null;
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
