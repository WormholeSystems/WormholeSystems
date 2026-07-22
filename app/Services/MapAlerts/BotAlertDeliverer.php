<?php

declare(strict_types=1);

namespace App\Services\MapAlerts;

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Models\MapAlert;
use App\Models\MapAlertDelivery;
use App\Services\Discord\DiscordBotDelivery;
use App\Services\Discord\DiscordConfigurationException;
use Illuminate\Http\Client\RequestException;
use Illuminate\Support\Facades\Gate;
use RuntimeException;
use Throwable;

/**
 * Shared bot-delivery pipeline for every alert evaluation job: verifies the creator
 * prerequisites, sends through the bot, and translates failures into either a disable
 * with a recorded reason (permanent) or a returned throwable (transient, retryable).
 */
final readonly class BotAlertDeliverer
{
    public function __construct(
        private DiscordBotDelivery $delivery,
        private MapAlertLifecycle $lifecycle,
    ) {}

    /**
     * Whether the alert's creator-side prerequisites still hold; disables the alert
     * with the matching reason when they do not.
     */
    public function passesCreatorPrerequisites(MapAlert $alert): bool
    {
        if ($alert->delivery_type === MapAlertDeliveryType::DiscordDm) {
            if ($alert->creator === null) {
                $this->lifecycle->disable($alert, null, MapAlertDisabledReason::DeliveryFailed, 'Discord direct message alert does not have a creator.');

                return false;
            }

            if ($alert->creator->discordAccount === null) {
                $this->lifecycle->disable($alert, null, MapAlertDisabledReason::DiscordAccountDisconnected);

                return false;
            }

            if (Gate::forUser($alert->creator)->denies('view', $alert->map)) {
                $this->lifecycle->disable($alert, null, MapAlertDisabledReason::AccessRevoked);

                return false;
            }

            return true;
        }

        if ($alert->requiresCreatorDiscordAccount()
            && ($alert->creator === null || $alert->creator->discordAccount === null)) {
            $this->lifecycle->disable($alert, null, MapAlertDisabledReason::DiscordAccountDisconnected);

            return false;
        }

        return true;
    }

    /**
     * Delivers the embed, marking the reservation and last firing on success. Permanent
     * failures disable the alert (keeping any reservation as the audit trail); transient
     * failures release the reservation and are reported and returned so the caller can
     * decide whether to retry the job.
     *
     * @param  array<string, mixed>  $embed
     */
    public function deliver(MapAlert $alert, array $embed, string $nonce, ?MapAlertDelivery $reservation = null): ?Throwable
    {
        try {
            $this->delivery->deliver($alert, $embed, $nonce);
            $reservation?->update(['delivered_at' => now()]);
            $alert->update(['last_fired_at' => now()]);

            return null;
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
                $this->lifecycle->disable($alert, null, $reason, $detail);

                return null;
            }

            return $this->transientFailure($exception, $reservation);
        } catch (DiscordConfigurationException $exception) {
            return $this->transientFailure($exception, $reservation);
        } catch (RuntimeException $exception) {
            $this->lifecycle->disable($alert, null, MapAlertDisabledReason::DeliveryFailed, $exception->getMessage());

            return null;
        } catch (Throwable $exception) {
            return $this->transientFailure($exception, $reservation);
        }
    }

    private function transientFailure(Throwable $exception, ?MapAlertDelivery $reservation): Throwable
    {
        $reservation?->delete();
        report($exception);

        return $exception;
    }
}
