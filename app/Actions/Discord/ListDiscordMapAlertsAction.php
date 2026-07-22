<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\MapAlert;
use Illuminate\Support\Facades\Gate;

final readonly class ListDiscordMapAlertsAction
{
    public function __construct(private DiscordAlertFormatter $formatter) {}

    public function handle(DiscordAccount $account, int $mapId): string
    {
        $map = Map::query()->find($mapId);
        if ($map === null || Gate::forUser($account->user)->denies('view', $map)) {
            return 'That map is unavailable.';
        }

        $isManager = Gate::forUser($account->user)->allows('manageAccess', $map);
        $alerts = $map->mapAlerts()
            ->when(! $isManager, fn ($query) => $query->shared())
            ->with(['creator:id,name', 'map:id,name', 'targetSolarsystem:id,name'])
            ->latest()
            ->get();

        return $alerts->isEmpty()
            ? 'This map has no visible alerts.'
            : $alerts->map(fn (MapAlert $alert): string => $this->formatter->format($alert, $isManager))->implode("\n");
    }
}
