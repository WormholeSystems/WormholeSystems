<?php

declare(strict_types=1);

namespace App\Actions\Discord\Concerns;

use App\Models\DiscordAccount;
use App\Models\MapAlert;
use Illuminate\Support\Facades\Gate;

trait ResolvesBotAlerts
{
    /**
     * Find a bot-delivered alert the account's user is allowed to act on, or null when it
     * does not exist or the user may not know it exists.
     */
    private function findAuthorizedBotAlert(DiscordAccount $account, int $alertId, string $ability): ?MapAlert
    {
        $alert = MapAlert::query()->bot()->with(['creator', 'map'])->find($alertId);
        if ($alert === null || Gate::forUser($account->user)->denies($ability, $alert)) {
            return null;
        }

        return $alert;
    }
}
