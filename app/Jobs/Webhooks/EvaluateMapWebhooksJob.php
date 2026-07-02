<?php

declare(strict_types=1);

namespace App\Jobs\Webhooks;

use App\Enums\MapWebhookType;
use App\Enums\SolarsystemClass;
use App\Models\MapAlert;
use App\Models\MapIgnoredSolarsystem;
use App\Models\Solarsystem;
use App\Services\Discord\DiscordDelivery;
use App\Services\Discord\JumpRangeAlertEmbed;
use App\Services\Discord\ProximityAlertEmbed;
use App\Services\JumpRange\JumpRangeCalculator;
use App\Services\Routing\MapProximityPathfinder;
use App\Services\Routing\ProximityResult;
use Illuminate\Contracts\Queue\ShouldBeUnique;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Support\Collection;

/**
 * Fired when a system is added to a map. Proximity alerts measure the k-space
 * (stargate) distance from the newly-added system to the alert's target; jump-range
 * alerts measure the light-year distance from a newly-added k-space exit to the
 * alert's target system. Each sends a single Discord alert when within range.
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

    public function handle(
        MapProximityPathfinder $pathfinder,
        ProximityAlertEmbed $proximityEmbed,
        JumpRangeAlertEmbed $jumpRangeEmbed,
        JumpRangeCalculator $calculator,
        DiscordDelivery $delivery,
    ): void {
        $alerts = MapAlert::query()
            ->where('map_id', $this->map_id)
            ->whereIn('type', [MapWebhookType::Proximity, MapWebhookType::JumpRange])
            ->where('is_active', true)
            ->with(['webhook', 'role'])
            ->get();

        if ($alerts->isEmpty()) {
            return;
        }

        $this->evaluateProximity($alerts->where('type', MapWebhookType::Proximity), $pathfinder, $proximityEmbed, $delivery);
        $this->evaluateJumpRange($alerts->where('type', MapWebhookType::JumpRange), $calculator, $jumpRangeEmbed, $delivery);
    }

    /**
     * @param  Collection<int, MapAlert>  $alerts
     */
    private function evaluateProximity(Collection $alerts, MapProximityPathfinder $pathfinder, ProximityAlertEmbed $embed, DiscordDelivery $delivery): void
    {
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

    /**
     * @param  Collection<int, MapAlert>  $alerts
     */
    private function evaluateJumpRange(Collection $alerts, JumpRangeCalculator $calculator, JumpRangeAlertEmbed $embed, DiscordDelivery $delivery): void
    {
        if ($alerts->isEmpty()) {
            return;
        }

        $exit = Solarsystem::query()->with('region:id,name')->find($this->solarsystem_id);

        if ($exit === null || $exit->type !== 'eve') {
            return;
        }

        $isHighsec = SolarsystemClass::fromSecurity((float) $exit->security) === SolarsystemClass::H;

        $targets = Solarsystem::query()
            ->whereIn('id', $alerts->pluck('target_solarsystem_id')->filter()->unique())
            ->get()
            ->keyBy('id');

        foreach ($alerts as $alert) {
            if ($isHighsec && ! $alert->include_highsec) {
                continue;
            }

            if ($alert->target_solarsystem_id === $exit->id) {
                continue;
            }

            $target = $targets[$alert->target_solarsystem_id] ?? null;

            if ($target === null || $alert->ship_type === null || $alert->jdc_level === null) {
                continue;
            }

            $distance = $calculator->distanceLy($exit, $target);

            if ($distance > $alert->ship_type->maxRangeLy($alert->jdc_level)) {
                continue;
            }

            $delivery->deliver($alert, $embed->build($alert, $exit, $target, $distance));

            $alert->update(['last_fired_at' => now()]);
        }
    }
}
