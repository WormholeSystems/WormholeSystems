<?php

declare(strict_types=1);

namespace App\Jobs\Webhooks;

use App\Enums\MapWebhookType;
use App\Models\MapAlert;
use App\Models\MapIgnoredSolarsystem;
use App\Services\Discord\DiscordDelivery;
use App\Services\Discord\ProximityAlertEmbed;
use App\Services\Routing\MapProximityPathfinder;
use App\Services\Routing\ProximityResult;
use Illuminate\Contracts\Queue\ShouldBeUnique;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;

/**
 * Fired when a system is added to a map. For each active webhook, it measures the
 * k-space (stargate) distance from the newly-added system to the webhook's target
 * and sends a single Discord alert if that system is within range.
 *
 * Because only the just-added system is checked, systems that already alerted are
 * never re-evaluated, so no webhook fires twice for the same system.
 */
final class EvaluateMapWebhooksJob implements ShouldBeUnique, ShouldQueue
{
    use Queueable;

    public int $uniqueFor = 30;

    public function __construct(
        public readonly int $map_id,
        public readonly int $solarsystem_id,
    ) {}

    public function uniqueId(): string
    {
        return $this->map_id.':'.$this->solarsystem_id;
    }

    public function handle(MapProximityPathfinder $pathfinder, ProximityAlertEmbed $embed, DiscordDelivery $delivery): void
    {
        $alerts = MapAlert::query()
            ->where('map_id', $this->map_id)
            ->where('type', MapWebhookType::Proximity)
            ->where('is_active', true)
            ->with(['webhook', 'role'])
            ->get();

        if ($alerts->isEmpty()) {
            return;
        }

        $ignored = MapIgnoredSolarsystem::query()
            ->where('map_id', $this->map_id)
            ->pluck('solarsystem_id')
            ->all();

        foreach ($alerts as $alert) {
            $result = $pathfinder->nearest(
                [$this->solarsystem_id],
                $alert->target_solarsystem_id,
                [],
                $ignored,
                $alert->max_jumps,
            );

            if (! $result instanceof ProximityResult) {
                continue;
            }

            $delivery->deliver($alert, $embed->build($alert, $result));

            $alert->update(['last_fired_at' => now()]);
        }
    }
}
