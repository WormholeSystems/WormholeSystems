<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

use App\Models\DiscordAccount;
use Discord\Parts\Interactions\ApplicationCommand;

interface CommandDefinition
{
    /**
     * The Discord application command this class registers.
     */
    public function definition(): SlashCommand;

    /**
     * Whether the interaction requires a linked Wormhole Systems account before
     * this command may respond.
     */
    public function requiresLinkedAccount(): bool;

    /**
     * The reply to an invocation of this command.
     */
    public function respond(ApplicationCommand $interaction, ?DiscordAccount $account): string;
}
