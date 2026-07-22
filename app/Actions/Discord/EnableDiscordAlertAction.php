<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Actions\Discord\Concerns\ResolvesBotAlerts;
use App\Actions\MapAlerts\EnableMapAlertAction;
use App\Models\DiscordAccount;

final readonly class EnableDiscordAlertAction
{
    use ResolvesBotAlerts;

    public function __construct(private EnableMapAlertAction $enableAlert) {}

    public function handle(DiscordAccount $account, int $alertId): string
    {
        $alert = $this->findAuthorizedBotAlert($account, $alertId, 'update');
        if (! $alert instanceof \App\Models\MapAlert) {
            return 'That alert does not exist.';
        }

        $error = $this->enableAlert->handle($alert, $account->user);

        return $error ?? 'Alert enabled.';
    }
}
