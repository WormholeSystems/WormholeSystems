<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Actions\Discord\Concerns\ResolvesBotAlerts;
use App\Models\DiscordAccount;
use App\Services\MapAlerts\MapAlertLifecycle;

final readonly class DeleteDiscordAlertAction
{
    use ResolvesBotAlerts;

    public function __construct(private MapAlertLifecycle $lifecycle) {}

    public function handle(DiscordAccount $account, int $alertId): string
    {
        $alert = $this->findAuthorizedBotAlert($account, $alertId, 'delete');
        if (! $alert instanceof \App\Models\MapAlert) {
            return 'That alert does not exist.';
        }

        $this->lifecycle->remove($alert, $account->user);

        return 'Alert removed.';
    }
}
