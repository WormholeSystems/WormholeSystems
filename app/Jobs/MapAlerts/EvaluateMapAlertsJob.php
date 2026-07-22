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
use App\Services\Discord\DiscordBotDelivery;
use App\Services\Discord\DiscordConfigurationException;
use App\Services\Discord\DiscordDelivery;
use App\Services\Discord\JumpRangeAlertEmbed;
use App\Services\Discord\PersonalProximityAlertEmbed;
use App\Services\Discord\ProximityAlertEmbed;
use App\Services\JumpRange\JumpRangeCalculator;
use App\Services\MapAlerts\MapAlertLifecycle;
use App\Services\Routing\MapProximityPathfinder;
use App\Services\Routing\ProximityResult;
use Illuminate\Contracts\Queue\ShouldBeUnique;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Http\Client\RequestException;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Gate;
use RuntimeException;
use Throwable;

/**
 * Fired when a system is added to a map. Evaluates every active alert regardless of how
 * it delivers: proximity alerts measure the k-space (stargate) distance from the
 * newly-added system to the alert's target; jump-range alerts measure the light-year
 * distance from a newly-added k-space exit to the alert's target system.
 *
 * Webhook alerts post once per added system (only the just-added system is checked, so
 * no webhook fires twice for it). Bot alerts additionally reserve a per-placement
 * delivery row so retries never double-send, and are disabled with a recorded reason
 * when their prerequisites or destination are gone.
 */
final class EvaluateMapAlertsJob implements ShouldBeUnique, ShouldQueue
{
    use Queueable;

    public int $uniqueFor = 60;

    public int $tries = 3;

    /** @var int[] */
    public array $backoff = [10, 60, 300];

    private ?Throwable $firstFailure = null;

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
        DiscordBotDelivery $botDelivery,
        MapAlertLifecycle $lifecycle,
    ): void {
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

        $reservations = $this->existingReservations($alerts, $placement);

        $this->evaluateProximity(
            $alerts->where('type', MapAlertType::Proximity),
            $placement,
            $reservations,
            $pathfinder,
            $proximityEmbed,
            $personalEmbed,
            $webhookDelivery,
            $botDelivery,
            $lifecycle,
        );
        $this->evaluateJumpRange(
            $alerts->where('type', MapAlertType::JumpRange)->where('delivery_type', MapAlertDeliveryType::Webhook),
            $placement,
            $reservations,
            $calculator,
            $jumpRangeEmbed,
            $webhookDelivery,
        );

        if ($this->firstFailure instanceof Throwable) {
            throw $this->firstFailure;
        }
    }

    /**
     * @param  Collection<int, MapAlert>  $alerts
     * @param  array<int, MapAlertDelivery>  $reservations
     */
    private function evaluateProximity(
        Collection $alerts,
        MapSolarsystem $placement,
        array $reservations,
        MapProximityPathfinder $pathfinder,
        ProximityAlertEmbed $proximityEmbed,
        PersonalProximityAlertEmbed $personalEmbed,
        DiscordDelivery $webhookDelivery,
        DiscordBotDelivery $botDelivery,
        MapAlertLifecycle $lifecycle,
    ): void {
        if ($alerts->isEmpty()) {
            return;
        }

        $ignored = MapIgnoredSolarsystem::query()
            ->where('map_id', $placement->map_id)
            ->pluck('solarsystem_id')
            ->all();

        foreach ($alerts as $alert) {
            $isBot = $alert->delivery_type->isBot();

            if ($isBot && ! $this->passesBotPrerequisites($alert, $placement, $lifecycle, $reservations)) {
                continue;
            }
            if ($alert->target_solarsystem_id === null) {
                continue;
            }
            if ($alert->max_jumps === null) {
                continue;
            }

            $result = $pathfinder->nearest([$placement->solarsystem_id], $alert->target_solarsystem_id, [], $ignored, $alert->max_jumps);
            if (! $result instanceof ProximityResult) {
                continue;
            }

            if (! $isBot) {
                $reservation = $this->claimReservation($alert, $placement, $reservations);
                if (! $reservation instanceof MapAlertDelivery) {
                    continue;
                }

                $webhookDelivery->deliver($alert, $proximityEmbed->build($alert, $result));
                $reservation->update(['delivered_at' => now()]);
                $alert->update(['last_fired_at' => now()]);

                continue;
            }

            $this->deliverBotAlert($alert, $placement, $result, $personalEmbed, $botDelivery, $lifecycle, $reservations);
        }
    }

    /**
     * @param  array<int, MapAlertDelivery>  $reservations
     */
    private function passesBotPrerequisites(MapAlert $alert, MapSolarsystem $placement, MapAlertLifecycle $lifecycle, array $reservations): bool
    {
        if ($alert->delivery_type === MapAlertDeliveryType::DiscordDm) {
            if ($alert->creator === null) {
                $this->reservation($alert, $placement, $reservations);
                $lifecycle->disable($alert, null, MapAlertDisabledReason::DeliveryFailed, 'Discord direct message alert does not have a creator.');

                return false;
            }

            if ($alert->creator->discordAccount === null) {
                $lifecycle->disable($alert, null, MapAlertDisabledReason::DiscordAccountDisconnected);

                return false;
            }

            if (Gate::forUser($alert->creator)->denies('view', $alert->map)) {
                $lifecycle->disable($alert, null, MapAlertDisabledReason::AccessRevoked);

                return false;
            }
        } elseif ($alert->requiresCreatorDiscordAccount()
            && ($alert->creator === null || $alert->creator->discordAccount === null)) {
            $lifecycle->disable($alert, null, MapAlertDisabledReason::DiscordAccountDisconnected);

            return false;
        }

        if ($alert->target_solarsystem_id === null || $alert->max_jumps === null) {
            $this->reservation($alert, $placement, $reservations);
            $lifecycle->disable($alert, null, MapAlertDisabledReason::DeliveryFailed, 'Discord proximity alert is missing its target or range.');

            return false;
        }

        return true;
    }

    /**
     * @param  array<int, MapAlertDelivery>  $reservations
     */
    private function deliverBotAlert(
        MapAlert $alert,
        MapSolarsystem $placement,
        ProximityResult $result,
        PersonalProximityAlertEmbed $embed,
        DiscordBotDelivery $delivery,
        MapAlertLifecycle $lifecycle,
        array $reservations,
    ): void {
        $reservation = $this->claimReservation($alert, $placement, $reservations);
        if (! $reservation instanceof MapAlertDelivery) {
            return;
        }

        try {
            $nonce = mb_substr(hash('sha256', $alert->id.':'.$placement->id), 0, 25);
            $delivery->deliver($alert, $embed->build($alert, $placement, $result), $nonce);
            $reservation->update(['delivered_at' => now()]);
            $alert->update(['last_fired_at' => now()]);
        } catch (RequestException $exception) {
            $status = $exception->response->status();
            [$reason, $detail] = match (true) {
                in_array($status, [403, 404], true) => [
                    MapAlertDisabledReason::DiscordDestinationUnavailable,
                    'Discord returned HTTP '.$status.' for the alert destination.',
                ],
                $status === 400 => [
                    MapAlertDisabledReason::DeliveryFailed,
                    'Discord rejected the alert payload with HTTP 400.',
                ],
                default => [null, null],
            };

            if ($reason !== null) {
                $lifecycle->disable($alert, null, $reason, $detail);

                return;
            }

            $reservation->delete();
            report($exception);
            $this->firstFailure ??= $exception;
        } catch (DiscordConfigurationException $exception) {
            $reservation->delete();
            report($exception);
            $this->firstFailure ??= $exception;
        } catch (RuntimeException $exception) {
            $lifecycle->disable($alert, null, MapAlertDisabledReason::DeliveryFailed, $exception->getMessage());
        } catch (Throwable $exception) {
            $reservation->delete();
            report($exception);
            $this->firstFailure ??= $exception;
        }
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
        if ($alerts->isEmpty()) {
            return [];
        }

        return MapAlertDelivery::query()
            ->whereIn('map_alert_id', $alerts->pluck('id'))
            ->where('map_solarsystem_id', $placement->id)
            ->get()
            ->keyBy('map_alert_id')
            ->all();
    }

    /**
     * @param  array<int, MapAlertDelivery>  $reservations
     */
    private function reservation(MapAlert $alert, MapSolarsystem $placement, array $reservations): MapAlertDelivery
    {
        return $reservations[$alert->id] ?? MapAlertDelivery::query()->createOrFirst([
            'map_alert_id' => $alert->id,
            'map_solarsystem_id' => $placement->id,
        ]);
    }

    /**
     * Claims this placement's delivery slot for the alert, or returns null when another
     * run already delivered it or holds a fresh claim. Keeps job retries from
     * double-sending any delivery type.
     *
     * @param  array<int, MapAlertDelivery>  $reservations
     */
    private function claimReservation(MapAlert $alert, MapSolarsystem $placement, array $reservations): ?MapAlertDelivery
    {
        $reservation = $this->reservation($alert, $placement, $reservations);
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

    /**
     * @param  Collection<int, MapAlert>  $alerts
     * @param  array<int, MapAlertDelivery>  $reservations
     */
    private function evaluateJumpRange(
        Collection $alerts,
        MapSolarsystem $placement,
        array $reservations,
        JumpRangeCalculator $calculator,
        JumpRangeAlertEmbed $embed,
        DiscordDelivery $delivery,
    ): void {
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
            if ($isHighsec && ! $alert->include_highsec) {
                continue;
            }

            if ($alert->target_solarsystem_id === $exit->id) {
                continue;
            }

            $target = $targets[$alert->target_solarsystem_id] ?? null;
            if ($target === null) {
                continue;
            }
            if ($alert->ship_type === null) {
                continue;
            }
            if ($alert->jdc_level === null) {
                continue;
            }

            $distance = $calculator->distanceLy($exit, $target);

            if ($distance > $alert->ship_type->maxRangeLy($alert->jdc_level)) {
                continue;
            }

            $reservation = $this->claimReservation($alert, $placement, $reservations);
            if (! $reservation instanceof MapAlertDelivery) {
                continue;
            }

            $delivery->deliver($alert, $embed->build($alert, $exit, $target, $distance));
            $reservation->update(['delivered_at' => now()]);
            $alert->update(['last_fired_at' => now()]);
        }
    }
}
