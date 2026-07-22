<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\MapIgnoredSolarsystem;
use App\Models\Solarsystem;
use App\Services\Routing\MapProximityPathfinder;
use App\Services\Routing\ProximityResult;
use Illuminate\Support\Facades\Gate;

final readonly class CalculateDiscordRouteAction
{
    public function __construct(private MapProximityPathfinder $pathfinder) {}

    public function handle(DiscordAccount $account, int $mapId, int $targetSolarsystemId, ?int $fromSolarsystemId = null): string
    {
        $map = Map::query()->find($mapId);
        $target = Solarsystem::query()->find($targetSolarsystemId);
        if ($map === null || $target === null || Gate::forUser($account->user)->denies('view', $map)) {
            return 'That map or target system is unavailable.';
        }

        $from = $fromSolarsystemId === null ? null : Solarsystem::query()->find($fromSolarsystemId);
        if ($fromSolarsystemId !== null && ! $from instanceof Solarsystem) {
            return 'That starting point system is unavailable.';
        }

        $origins = $from instanceof Solarsystem
            ? [$from->id]
            : $map->mapSolarsystems()->pluck('solarsystem_id')->all();
        $edges = $map->wormholeEdges();
        $ignored = MapIgnoredSolarsystem::query()->where('map_id', $map->id)->pluck('solarsystem_id')->all();
        $result = $this->pathfinder->nearest($origins, $target->id, $edges, $ignored, 100);
        if (! $result instanceof ProximityResult) {
            return 'No route was found within 100 jumps.';
        }

        $names = Solarsystem::query()->whereIn('id', $result->route)->pluck('name', 'id');
        $route = array_map(fn (int $id): string => $names[$id] ?? (string) $id, $result->route);

        return sprintf('**%d jumps**: %s', $result->jumps, implode(' -> ', $route));
    }
}
