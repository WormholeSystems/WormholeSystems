<?php

declare(strict_types=1);

namespace App\Jobs\Webhooks;

use App\Enums\KillmailFilterSubject;
use App\Enums\MapWebhookType;
use App\Models\Killmail;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\MapIgnoredSolarsystem;
use App\Models\MapSolarsystem;
use App\Models\Type;
use App\Services\Discord\DiscordDelivery;
use App\Services\Discord\KillmailAlertEmbed;
use App\Services\Killmails\KillmailWebhookMatcher;
use App\Services\Routing\MapProximityPathfinder;
use App\Services\Routing\ProximityResult;
use Illuminate\Contracts\Queue\ShouldBeUnique;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Support\Collection;

/**
 * Evaluates every active killmail webhook against a freshly-ingested killmail. For each
 * webhook it first checks the (cheap) filter rules, then the (more expensive) range from
 * the map to the kill's system, and sends a single Discord alert when both pass.
 *
 * Dispatched for every stored killmail, so it returns immediately when no killmail
 * webhooks exist — the common case for the global zKillboard firehose.
 */
final class EvaluateKillmailWebhooksJob implements ShouldBeUnique, ShouldQueue
{
    use Queueable;

    public int $uniqueFor = 120;

    public function __construct(
        public readonly int $killmail_id,
    ) {}

    public function uniqueId(): string
    {
        return 'killmail:'.$this->killmail_id;
    }

    public function handle(MapProximityPathfinder $pathfinder, KillmailAlertEmbed $embed, DiscordDelivery $delivery, KillmailWebhookMatcher $matcher): void
    {
        $alerts = MapAlert::query()
            ->where('type', MapWebhookType::Killmail)
            ->where('is_active', true)
            ->with(['webhook', 'role'])
            ->get();

        if ($alerts->isEmpty()) {
            return;
        }

        $killmail = Killmail::query()->find($this->killmail_id);

        if ($killmail === null) {
            return;
        }

        /** @var array<string, mixed> $data */
        $data = json_decode(json_encode($killmail->data), true) ?: [];

        $typeGroupMap = $this->resolveTypeGroupMap($alerts, $data);
        $mapData = $this->loadMapData($alerts->pluck('map_id')->unique()->all());

        // Index the killmail once; every alert's filters query the same pools.
        $pools = $matcher->buildPools($data, $typeGroupMap);

        foreach ($alerts as $alert) {
            if (! $matcher->matches($pools, $alert->filters, $alert->filter_match)) {
                continue;
            }

            $map = $mapData[$alert->map_id] ?? null;
            if ($map === null) {
                continue;
            }
            if ($map['systems'] === []) {
                continue;
            }

            $result = $pathfinder->nearest(
                $map['systems'],
                $killmail->solarsystem_id,
                $map['edges'],
                $map['ignored'],
                $alert->max_jumps,
            );

            if (! $result instanceof ProximityResult) {
                continue;
            }

            $matchedFilters = $matcher->matchingRules($pools, $alert->filters);

            $delivery->deliver($alert, $embed->build($alert, $killmail, $result, $matchedFilters));

            $alert->update(['last_fired_at' => now()]);
        }
    }

    /**
     * Resolve ship_type_id => group_id for the kill's ships, but only when an alert
     * actually filters on ship group.
     *
     * @param  Collection<int, MapAlert>  $alerts
     * @param  array<string, mixed>  $data
     * @return array<int, int>
     */
    private function resolveTypeGroupMap(Collection $alerts, array $data): array
    {
        $needsGroups = $alerts->contains(
            fn (MapAlert $alert): bool => $alert->filters->contains(
                fn ($rule): bool => $rule->subject === KillmailFilterSubject::ShipGroup,
            ),
        );

        if (! $needsGroups) {
            return [];
        }

        $shipTypeIds = collect([$data['victim']['ship_type_id'] ?? null])
            ->merge(collect($data['attackers'] ?? [])->pluck('ship_type_id'))
            ->filter()
            ->map(intval(...))
            ->unique()
            ->all();

        if ($shipTypeIds === []) {
            return [];
        }

        return Type::query()
            ->whereIn('id', $shipTypeIds)
            ->pluck('group_id', 'id')
            ->all();
    }

    /**
     * Cache each map's system ids, ignored ids and wormhole edges once per run.
     *
     * @param  int[]  $mapIds
     * @return array<int, array{systems: int[], ignored: int[], edges: array<int, array{0: int, 1: int}>}>
     */
    private function loadMapData(array $mapIds): array
    {
        $maps = Map::query()
            ->whereIn('id', $mapIds)
            ->with([
                'mapConnections.fromMapSolarsystem:id,solarsystem_id',
                'mapConnections.toMapSolarsystem:id,solarsystem_id',
            ])
            ->get();

        $systemsByMap = MapSolarsystem::query()
            ->whereIn('map_id', $mapIds)
            ->get(['map_id', 'solarsystem_id'])
            ->groupBy('map_id');

        $ignoredByMap = MapIgnoredSolarsystem::query()
            ->whereIn('map_id', $mapIds)
            ->get(['map_id', 'solarsystem_id'])
            ->groupBy('map_id');

        $data = [];

        foreach ($maps as $map) {
            $edges = $map->mapConnections
                ->map(fn ($connection): array => [$connection->fromMapSolarsystem->solarsystem_id, $connection->toMapSolarsystem->solarsystem_id])
                ->values()
                ->all();

            $data[$map->id] = [
                'systems' => ($systemsByMap[$map->id] ?? collect())->pluck('solarsystem_id')->all(),
                'ignored' => ($ignoredByMap[$map->id] ?? collect())->pluck('solarsystem_id')->all(),
                'edges' => $edges,
            ];
        }

        return $data;
    }
}
