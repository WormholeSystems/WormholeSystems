<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Models\DiscordAccount;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Str;

final readonly class CreateDiscordAccountLinkAction
{
    /** @param array{id: string, username: string, global_name: string|null, avatar: string|null} $identity */
    public function handle(?DiscordAccount $account, array $identity): string
    {
        if ($account instanceof DiscordAccount) {
            return 'Your Discord account is already linked.';
        }

        $token = Str::random(48);
        Cache::put('discord.link.'.$token, $identity, now()->addMinutes(15));

        return 'Link your account within 15 minutes: '.route('discord.link', $token);
    }
}
