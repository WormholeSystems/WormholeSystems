<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

final readonly class CommandRegistry
{
    public function __construct(
        private AccountCommand $account,
        private AlertCommand $alert,
        private AlertsCommand $alerts,
        private RouteCommand $route,
    ) {}

    /** @return CommandDefinition[] */
    public function all(): array
    {
        return [$this->account, $this->alert, $this->alerts, $this->route];
    }
}
