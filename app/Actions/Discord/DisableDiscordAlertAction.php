<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Actions\Discord\Concerns\ResolvesBotAlerts;
use App\Enums\MapAlertDisabledReason;
use App\Models\DiscordAccount;
use App\Services\MapAlerts\MapAlertLifecycle;

final readonly class DisableDiscordAlertAction
{
    use ResolvesBotAlerts;

    public function __construct(private MapAlertLifecycle $lifecycle) {}

    public function handle(DiscordAccount $account, int $alertId): string
    {
        $alert = $this->findAuthorizedBotAlert($account, $alertId, 'update');
        if (! $alert instanceof \App\Models\MapAlert) {
            return 'That alert does not exist.';
        }

        $this->lifecycle->disable($alert, $account->user, MapAlertDisabledReason::Manual);

        return 'Alert disabled.';
    }
}
