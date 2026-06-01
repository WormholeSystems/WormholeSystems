<?php

declare(strict_types=1);

namespace App\Services;

use App\Models\MapWebhook;
use App\Models\Solarsystem;
use App\Services\Routing\ProximityResult;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Throwable;

final class DiscordWebhookService
{
    private const int COLOR_HIGHSEC = 0x2ECC71;

    private const int COLOR_LOWSEC = 0xE67E22;

    private const int COLOR_NULLSEC = 0xE74C3C;

    /**
     * POST a proximity alert embed to the webhook's Discord URL, coloured by the
     * target system's security status.
     */
    public function sendProximityAlert(MapWebhook $webhook, ProximityResult $result): void
    {
        $systems = $this->resolveSystems($webhook, $result);

        $target = $systems[$webhook->target_solarsystem_id] ?? ['name' => (string) $webhook->target_solarsystem_id, 'security' => 0.0];
        $originName = $systems[$result->matchedOriginSolarsystemId]['name'] ?? (string) $result->matchedOriginSolarsystemId;
        $routeNames = array_map(fn (int $id): string => $systems[$id]['name'] ?? (string) $id, $result->route);

        try {
            Http::timeout(10)
                ->retry(3, 200)
                ->post($webhook->discord_webhook_url, [
                    'embeds' => [
                        [
                            'title' => sprintf('Found new %d %s %s connection', $result->jumps, $result->jumps === 1 ? 'jump' : 'jumps', $target['name']),
                            'description' => sprintf('**%s** was just added to your map, putting **%s** within range.', $originName, $target['name']),
                            'color' => $this->colorForSecurity((float) $target['security']),
                            'fields' => [
                                ['name' => 'Target', 'value' => sprintf('%s (%.1f)', $target['name'], $target['security']), 'inline' => true],
                                ['name' => 'Gate jumps', 'value' => (string) $result->jumps, 'inline' => true],
                                ['name' => 'From system', 'value' => $originName, 'inline' => true],
                                ['name' => 'Route', 'value' => implode(' → ', $routeNames)],
                            ],
                        ],
                    ],
                ])
                ->throw();
        } catch (Throwable $e) {
            Log::warning('Discord webhook delivery failed', [
                'map_webhook_id' => $webhook->id,
                'message' => $e->getMessage(),
            ]);
        }
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
    private function resolveSystems(MapWebhook $webhook, ProximityResult $result): array
    {
        $ids = array_unique([
            $webhook->target_solarsystem_id,
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
