<?php

declare(strict_types=1);

namespace App\Services;

use App\Models\Killmail;
use App\Models\MapAlert;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use App\Models\Type;
use App\Services\Killmails\KillmailFilterDescriber;
use App\Services\Killmails\KillmailFilterRule;
use App\Services\Routing\ProximityResult;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Throwable;

final readonly class DiscordWebhookService
{
    private const int COLOR_HIGHSEC = 0x2ECC71;

    private const int COLOR_LOWSEC = 0xE67E22;

    private const int COLOR_NULLSEC = 0xE74C3C;

    public function __construct(
        private KillmailFilterDescriber $filterDescriber,
    ) {}

    /**
     * POST a proximity alert embed to the alert's webhook URL, coloured by the
     * target system's security status.
     */
    public function sendProximityAlert(MapAlert $alert, ProximityResult $result): void
    {
        $systems = $this->resolveSystems($alert, $result);

        $target = $systems[$alert->target_solarsystem_id] ?? ['name' => (string) $alert->target_solarsystem_id, 'security' => 0.0];
        $originName = $systems[$result->matchedOriginSolarsystemId]['name'] ?? (string) $result->matchedOriginSolarsystemId;
        $routeNames = array_map(fn (int $id): string => $systems[$id]['name'] ?? (string) $id, $result->route);

        $this->deliver($alert, [
            'title' => sprintf('Found new %d %s %s connection', $result->jumps, $result->jumps === 1 ? 'jump' : 'jumps', $target['name']),
            'description' => sprintf('**%s** was just added to your map, putting **%s** within range.', $originName, $target['name']),
            'color' => $this->colorForSecurity((float) $target['security']),
            'fields' => [
                ['name' => 'Target', 'value' => sprintf('%s (%.1f)', $target['name'], $target['security']), 'inline' => true],
                ['name' => 'Gate jumps', 'value' => (string) $result->jumps, 'inline' => true],
                ['name' => 'From system', 'value' => $originName, 'inline' => true],
                ['name' => 'Route', 'value' => implode(' → ', $routeNames)],
            ],
        ]);
    }

    /**
     * POST a killmail alert embed to the alert's webhook URL when a kill matching the
     * alert's filters occurs within range of the map.
     *
     * @param  Collection<int, KillmailFilterRule>  $matchedFilters
     */
    public function sendKillmailAlert(MapAlert $alert, Killmail $killmail, ProximityResult $result, Collection $matchedFilters): void
    {
        $data = json_decode(json_encode($killmail->data), true) ?: [];
        $victim = is_array($data['victim'] ?? null) ? $data['victim'] : [];
        $shipTypeId = (int) ($victim['ship_type_id'] ?? 0);

        $systems = Solarsystem::query()
            ->whereIn('id', $result->route)
            ->get(['id', 'name', 'security'])
            ->keyBy('id');

        $systemName = $systems[$killmail->solarsystem_id]->name ?? (string) $killmail->solarsystem_id;
        $security = (float) ($systems[$killmail->solarsystem_id]->security ?? 0.0);

        $shipName = $this->typeName($shipTypeId);
        $totalValue = (float) (data_get($killmail->zkb, 'totalValue') ?? 0.0);

        $fields = [
            ['name' => 'System', 'value' => sprintf('%s (%.1f)', $systemName, $security), 'inline' => true],
            ['name' => 'Ship', 'value' => $shipName, 'inline' => true],
            ['name' => 'Value', 'value' => sprintf('%s ISK', number_format($totalValue)), 'inline' => true],
        ];

        if ($killmail->time !== null) {
            $fields[] = ['name' => 'When', 'value' => sprintf('<t:%d:R>', $killmail->time->timestamp), 'inline' => true];
        }

        if ($result->jumps === 0) {
            $description = 'A matching killmail occurred **inside your chain**.';
        } else {
            $description = sprintf('A matching killmail occurred **%d %s** from your chain.', $result->jumps, $result->jumps === 1 ? 'jump' : 'jumps');
            $fields[] = ['name' => 'Exit from', 'value' => $this->resolveExit($alert, $result, $systems), 'inline' => true];
            $fields[] = ['name' => 'Jumps from chain', 'value' => (string) $result->jumps, 'inline' => true];
            $fields[] = ['name' => 'Route', 'value' => implode(' → ', array_map(fn (int $id): string => $systems[$id]->name ?? (string) $id, $result->route))];
        }

        $filterLines = $this->filterDescriber->describe($matchedFilters);

        if ($filterLines !== []) {
            $fields[] = ['name' => 'Matched filters', 'value' => implode("\n", $filterLines)];
        }

        $embed = [
            'title' => sprintf('%s killed in %s', $shipName, $systemName),
            'url' => sprintf('https://zkillboard.com/kill/%d/', $killmail->id),
            'description' => $description,
            'color' => $this->colorForSecurity($security),
            'fields' => $fields,
        ];

        if ($killmail->time !== null) {
            $embed['timestamp'] = $killmail->time->toIso8601String();
        }

        if ($shipTypeId > 0) {
            $embed['thumbnail'] = ['url' => sprintf('https://images.evetech.net/types/%d/render?size=128', $shipTypeId)];
        }

        $this->deliver($alert, $embed);
    }

    /**
     * POST a single embed to the alert's webhook URL, pinging the configured role when
     * one is set. Delivery failures are logged but never bubble up.
     *
     * @param  array<string, mixed>  $embed
     */
    private function deliver(MapAlert $alert, array $embed): void
    {
        $payload = ['embeds' => [$embed]];

        $roleId = $alert->role?->discord_role_id;

        if ($roleId !== null) {
            $payload['content'] = sprintf('<@&%s>', $roleId);
            $payload['allowed_mentions'] = ['roles' => [$roleId]];
        }

        try {
            Http::timeout(10)
                ->retry(3, 200)
                ->post($alert->webhook->discord_webhook_url, $payload)
                ->throw();
        } catch (Throwable $e) {
            Log::warning('Discord webhook delivery failed', [
                'map_alert_id' => $alert->id,
                'message' => $e->getMessage(),
            ]);
        }
    }

    /**
     * The chain system nearest the kill — where a pilot would exit the map — shown with
     * its map alias when one is set.
     *
     * @param  Collection<int, Solarsystem>  $systems
     */
    private function resolveExit(MapAlert $alert, ProximityResult $result, Collection $systems): string
    {
        $originId = $result->matchedOriginSolarsystemId;
        $name = $systems[$originId]->name ?? (string) $originId;

        $alias = MapSolarsystem::query()
            ->where('map_id', $alert->map_id)
            ->isSolarsystem($originId)
            ->value('alias');

        return $alias ? sprintf('%s (%s)', $alias, $name) : $name;
    }

    private function typeName(int $typeId): string
    {
        if ($typeId === 0) {
            return 'Unknown ship';
        }

        return Type::query()->find($typeId)?->name ?? (string) $typeId;
    }

    private function colorForSecurity(float $security): int
    {
        return match (true) {
            $security >= 0.45 => self::COLOR_HIGHSEC,
            $security > 0.0 => self::COLOR_LOWSEC,
            default => self::COLOR_NULLSEC,
        };
    }

    /**
     * @return array<int, array{name: string, security: float}>
     */
    private function resolveSystems(MapAlert $alert, ProximityResult $result): array
    {
        $ids = array_unique([
            $alert->target_solarsystem_id,
            $result->matchedOriginSolarsystemId,
            ...$result->route,
        ]);

        return Solarsystem::query()
            ->whereIn('id', $ids)
            ->get(['id', 'name', 'security'])
            ->keyBy('id')
            ->map(fn (Solarsystem $system): array => ['name' => $system->name, 'security' => (float) $system->security])
            ->all();
    }
}
