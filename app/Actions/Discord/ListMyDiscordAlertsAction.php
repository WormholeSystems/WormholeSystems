<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Models\DiscordAccount;
use App\Models\MapAlert;

final readonly class ListMyDiscordAlertsAction
{
    public function __construct(private DiscordAlertFormatter $formatter) {}

    public function handle(DiscordAccount $account): string
    {
        $alerts = $account->user->createdMapAlerts()->bot()->with(['map:id,name', 'targetSolarsystem:id,name', 'originSolarsystem:id,name'])->latest()->get();

        return $alerts->isEmpty()
            ? 'You have no proximity alerts.'
            : $alerts->map(fn (MapAlert $alert): string => $this->formatter->format($alert))->implode("\n");
    }
}
