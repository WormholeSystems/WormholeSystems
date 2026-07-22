<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

use App\Actions\Discord\CreateDiscordAccountLinkAction;
use App\Actions\Discord\GetDiscordAccountStatusAction;
use App\Models\DiscordAccount;
use Discord\Parts\Interactions\ApplicationCommand;

final readonly class AccountCommand implements CommandDefinition
{
    use ReadsInteractionOptions;

    public function __construct(
        private GetDiscordAccountStatusAction $getAccountStatus,
        private CreateDiscordAccountLinkAction $createAccountLink,
    ) {}

    public function definition(): SlashCommand
    {
        return SlashCommand::make('account', 'Manage your linked Wormhole Systems account')->options(
            SubCommand::make('link', 'Link this Discord account'),
            SubCommand::make('status', 'Show your account link status'),
        );
    }

    public function requiresLinkedAccount(): bool
    {
        return false;
    }

    public function respond(ApplicationCommand $interaction, ?DiscordAccount $account): string
    {
        return match ($this->subcommand($interaction)->name) {
            'status' => $this->getAccountStatus->handle($account),
            'link' => $this->createAccountLink->handle($account, $this->identity($interaction)),
            default => 'That account command is unavailable.',
        };
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
}
