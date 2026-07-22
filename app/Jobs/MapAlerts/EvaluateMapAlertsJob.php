<?php

declare(strict_types=1);

namespace App\Jobs\MapAlerts;

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertType;
use App\Enums\SolarsystemClass;
use App\Models\MapAlert;
use App\Models\MapAlertDelivery;
use App\Models\MapIgnoredSolarsystem;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use App\Services\Discord\DiscordDelivery;
use App\Services\Discord\JumpRangeAlertEmbed;
use App\Services\Discord\PersonalProximityAlertEmbed;
use App\Services\Discord\ProximityAlertEmbed;
use App\Services\JumpRange\JumpRangeCalculator;
use App\Services\MapAlerts\BotAlertDeliverer;
use App\Services\MapAlerts\MapAlertLifecycle;
use App\Services\Routing\MapProximityPathfinder;
use App\Services\Routing\ProximityResult;
use Illuminate\Contracts\Queue\ShouldBeUnique;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Support\Collection;
use Throwable;

/**
 * Fired when a system is added to a map. Evaluates every active alert regardless of how
 * it delivers: proximity alerts measure the k-space (stargate) distance from the
 * newly-added system to the alert's target; jump-range alerts measure the light-year
 * distance from a newly-added k-space exit to the alert's target system.
 *
 * Every delivery reserves a per-placement row first, so job retries never double-send.
 * Bot alerts are additionally disabled with a recorded reason when their prerequisites
 * or destination are gone.
 */
final class EvaluateMapAlertsJob implements ShouldBeUnique, ShouldQueue
{
    use Queueable;

    public int $uniqueFor = 60;

    public int $tries = 3;

    /** @var int[] */
    public array $backoff = [10, 60, 300];

    private ?Throwable $firstFailure = null;

    private MapProximityPathfinder $pathfinder;

    private ProximityAlertEmbed $proximityEmbed;

    private PersonalProximityAlertEmbed $personalEmbed;

    private JumpRangeAlertEmbed $jumpRangeEmbed;

    private JumpRangeCalculator $calculator;

    private DiscordDelivery $webhookDelivery;

    private BotAlertDeliverer $botDeliverer;

    private MapAlertLifecycle $lifecycle;

    /** @var array<int, MapAlertDelivery> */
    private array $reservations = [];

    public function __construct(public readonly int $map_solarsystem_id) {}

    public function uniqueId(): string
    {
        return (string) $this->map_solarsystem_id;
    }

    public function handle(
        MapProximityPathfinder $pathfinder,
        ProximityAlertEmbed $proximityEmbed,
        PersonalProximityAlertEmbed $personalEmbed,
        JumpRangeAlertEmbed $jumpRangeEmbed,
        JumpRangeCalculator $calculator,
        DiscordDelivery $webhookDelivery,
        BotAlertDeliverer $botDeliverer,
        MapAlertLifecycle $lifecycle,
    ): void {
        $this->pathfinder = $pathfinder;
        $this->proximityEmbed = $proximityEmbed;
        $this->personalEmbed = $personalEmbed;
        $this->jumpRangeEmbed = $jumpRangeEmbed;
        $this->calculator = $calculator;
        $this->webhookDelivery = $webhookDelivery;
        $this->botDeliverer = $botDeliverer;
        $this->lifecycle = $lifecycle;

        $placement = MapSolarsystem::query()->with('map')->find($this->map_solarsystem_id);
        if ($placement === null) {
            return;
        }

        $alerts = MapAlert::query()
            ->where('map_id', $placement->map_id)
            ->whereIn('type', [MapAlertType::Proximity, MapAlertType::JumpRange])
            ->where('is_active', true)
            ->with(['webhook', 'role', 'creator.discordAccount', 'creator.characters', 'map'])
            ->get();

        if ($alerts->isEmpty()) {
            return;
        }

        $this->reservations = $this->existingReservations($alerts, $placement);

        $this->evaluateProximity($alerts->where('type', MapAlertType::Proximity), $placement);
        $this->evaluateJumpRange($alerts->where('type', MapAlertType::JumpRange), $placement);

        if ($this->firstFailure instanceof Throwable) {
            throw $this->firstFailure;
        }
    }

    /**
     * @param  Collection<int, MapAlert>  $alerts
     */
    private function evaluateProximity(Collection $alerts, MapSolarsystem $placement): void
    {
        if ($alerts->isEmpty()) {
            return;
        }

        $ignored = MapIgnoredSolarsystem::query()
            ->where('map_id', $placement->map_id)
            ->pluck('solarsystem_id')
            ->all();

        foreach ($alerts as $alert) {
            $this->evaluateProximityAlert($alert, $placement, $ignored);
        }
    }

    /**
     * @param  int[]  $ignored
     */
    private function evaluateProximityAlert(MapAlert $alert, MapSolarsystem $placement, array $ignored): void
    {
        $isBot = $alert->delivery_type->isBot();

        if ($isBot && ! $this->passesBotPrerequisites($alert, $placement)) {
            return;
        }

        if ($alert->target_solarsystem_id === null || $alert->max_jumps === null) {
            $this->disableMisconfiguredBotAlert($alert, $placement, 'Discord proximity alert is missing its target or range.');

            return;
        }

        $result = $this->pathfinder->nearest([$placement->solarsystem_id], $alert->target_solarsystem_id, [], $ignored, $alert->max_jumps);
        if (! $result instanceof ProximityResult) {
            return;
        }

        $embed = $isBot
            ? $this->personalEmbed->build($alert, $placement, $result)
            : $this->proximityEmbed->build($alert, $result);

        $this->deliver($alert, $placement, $embed);
    }

    /**
     * @param  Collection<int, MapAlert>  $alerts
     */
    private function evaluateJumpRange(Collection $alerts, MapSolarsystem $placement): void
    {
        if ($alerts->isEmpty()) {
            return;
        }

        $exit = Solarsystem::query()->with('region:id,name')->find($placement->solarsystem_id);

        if ($exit === null || $exit->type !== 'eve') {
            return;
        }

        $isHighsec = SolarsystemClass::fromSecurity((float) $exit->security) === SolarsystemClass::H;

        $targets = Solarsystem::query()
            ->whereIn('id', $alerts->pluck('target_solarsystem_id')->filter()->unique())
            ->get()
            ->keyBy('id');

        foreach ($alerts as $alert) {
            $this->evaluateJumpRangeAlert($alert, $placement, $exit, $isHighsec, $targets[$alert->target_solarsystem_id] ?? null);
        }
    }

    private function evaluateJumpRangeAlert(
        MapAlert $alert,
        MapSolarsystem $placement,
        Solarsystem $exit,
        bool $isHighsec,
        ?Solarsystem $target,
    ): void {
        $isBot = $alert->delivery_type->isBot();

        if ($isBot && ! $this->passesBotPrerequisites($alert, $placement)) {
            return;
        }

        if ($alert->target_solarsystem_id === null || $alert->ship_type === null || $alert->jdc_level === null) {
            $this->disableMisconfiguredBotAlert($alert, $placement, 'Discord jump-range alert is missing its target, ship, or calibration.');

            return;
        }

        if ($isHighsec && ! $alert->include_highsec) {
            return;
        }

        if ($alert->target_solarsystem_id === $exit->id) {
            return;
        }

        if (! $target instanceof Solarsystem) {
            return;
        }

        $distance = $this->calculator->distanceLy($exit, $target);

        if ($distance > $alert->ship_type->maxRangeLy($alert->jdc_level)) {
            return;
        }

        $this->deliver($alert, $placement, $this->jumpRangeEmbed->build($alert, $exit, $target, $distance));
    }

    /**
     * Creator-side prerequisites for bot alerts; a direct message alert without any
     * creator also records its reservation so the placement stays marked as handled.
     */
    private function passesBotPrerequisites(MapAlert $alert, MapSolarsystem $placement): bool
    {
        if ($alert->delivery_type === MapAlertDeliveryType::DiscordDm && $alert->creator === null) {
            $this->reservation($alert, $placement);
        }

        return $this->botDeliverer->passesCreatorPrerequisites($alert);
    }

    private function disableMisconfiguredBotAlert(MapAlert $alert, MapSolarsystem $placement, string $detail): void
    {
        if (! $alert->delivery_type->isBot()) {
            return;
        }

        $this->reservation($alert, $placement);
        $this->lifecycle->disable($alert, null, MapAlertDisabledReason::DeliveryFailed, $detail);
    }

    /**
     * Claims the placement's delivery slot and sends through the alert's channel.
     *
     * @param  array<string, mixed>  $embed
     */
    private function deliver(MapAlert $alert, MapSolarsystem $placement, array $embed): void
    {
        $reservation = $this->claimReservation($alert, $placement);
        if (! $reservation instanceof MapAlertDelivery) {
            return;
        }

        if (! $alert->delivery_type->isBot()) {
            $this->webhookDelivery->deliver($alert, $embed);
            $reservation->update(['delivered_at' => now()]);
            $alert->update(['last_fired_at' => now()]);

            return;
        }

        $nonce = mb_substr(hash('sha256', $alert->id.':'.$placement->id), 0, 25);
        $failure = $this->botDeliverer->deliver($alert, $embed, $nonce, $reservation);
        $this->firstFailure ??= $failure;
    }

    /**
     * Preloads every alert's delivery reservation for this placement in one query;
     * missing rows are inserted race-safely when an alert actually fires.
     *
     * @param  Collection<int, MapAlert>  $alerts
     * @return array<int, MapAlertDelivery>
     */
    private function existingReservations(Collection $alerts, MapSolarsystem $placement): array
    {
        return MapAlertDelivery::query()
            ->whereIn('map_alert_id', $alerts->pluck('id'))
            ->where('map_solarsystem_id', $placement->id)
            ->get()
            ->keyBy('map_alert_id')
            ->all();
    }

    private function reservation(MapAlert $alert, MapSolarsystem $placement): MapAlertDelivery
    {
        return $this->reservations[$alert->id] ?? MapAlertDelivery::query()->createOrFirst([
            'map_alert_id' => $alert->id,
            'map_solarsystem_id' => $placement->id,
        ]);
    }

    /**
     * Claims this placement's delivery slot for the alert, or returns null when another
     * run already delivered it or holds a fresh claim. Keeps job retries from
     * double-sending any delivery type.
     */
    private function claimReservation(MapAlert $alert, MapSolarsystem $placement): ?MapAlertDelivery
    {
        $reservation = $this->reservation($alert, $placement);
        if (! $reservation->wasRecentlyCreated && $reservation->delivered_at !== null) {
            return null;
        }

        if (! $reservation->wasRecentlyCreated) {
            $claimed = MapAlertDelivery::query()
                ->whereKey($reservation->id)
                ->whereNull('delivered_at')
                ->where('updated_at', '<=', now()->subMinutes(10))
                ->update(['updated_at' => now()]);

            if ($claimed === 0) {
                return null;
            }
        }

        return $reservation;
    }
}
