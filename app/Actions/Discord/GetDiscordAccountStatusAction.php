<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Models\DiscordAccount;

final readonly class GetDiscordAccountStatusAction
{
    public function handle(?DiscordAccount $account): string
    {
        return $account instanceof DiscordAccount
            ? 'Linked to Wormhole Systems as **'.$account->user->alertDisplayName().'**.'
            : 'Your Discord account is not linked.';
    }
}
