<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Data\MapConnectionData;
use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Events\MapConnections\MapConnectionUpdatedEvent;
use App\Models\MapConnection;
use Illuminate\Support\Facades\DB;
use Spatie\LaravelData\Optional;
use Throwable;

final class UpdateMapConnectionAction
{
    /**
     * @throws Throwable
     */
    public function handle(MapConnection $mapConnection, MapConnectionData $data): MapConnection
    {
        return DB::transaction(function () use ($mapConnection, $data): MapConnection {

            $mapConnection->update($data->toArray());

            $this->syncMassAndLifetime($mapConnection, $data);

            broadcast(new MapConnectionUpdatedEvent($mapConnection->map_id))->toOthers();

            return $mapConnection;
        });
    }

    private function syncMassAndLifetime(MapConnection $mapConnection, MapConnectionData $data): void
    {
        $signatures = $mapConnection->signatures;

        if ($signatures->isEmpty()) {
            return;
        }

        $lifetime = $this->getNewLifetimeValue($mapConnection, $data);
        $mass = $this->getNewMassStatus($mapConnection, $data);

        $updateData = ['mass_status' => $mass];

        // Only update lifetime and timestamp if lifetime actually changed
        if ($mapConnection->lifetime !== $lifetime) {
            $updateData['lifetime'] = $lifetime;
            $updateData['lifetime_updated_at'] = now();
        }

        $mapConnection->update($updateData);

        foreach ($signatures as $signature) {
            $signatureUpdateData = ['mass_status' => $mass];

            // Only update lifetime and timestamp if lifetime actually changed
            if ($signature->lifetime !== $lifetime) {
                $signatureUpdateData['lifetime'] = $lifetime;
                $signatureUpdateData['lifetime_updated_at'] = now();
            }

            $signature->update($signatureUpdateData);
        }
    }

    private function getMassStatusSeverity(MassStatus $status): int
    {
        return match ($status) {
            MassStatus::Fresh => 1,
            MassStatus::Reduced => 2,
            MassStatus::Critical => 3,
            MassStatus::Unknown => 0,
        };
    }

    private function getNewMassStatus(MapConnection $mapConnection, MapConnectionData $data): ?MassStatus
    {
        if (! $data->mass_status instanceof Optional) {
            return $data->mass_status;
        }

        $signatures = $mapConnection->signatures;
        if ($signatures->isEmpty()) {
            return $mapConnection->mass_status;
        }

        $conn_mass_severity = $mapConnection->mass_status ? $this->getMassStatusSeverity($mapConnection->mass_status) : 0;
        $max_severity = $conn_mass_severity;
        $worst_mass_status = $mapConnection->mass_status;

        foreach ($signatures as $signature) {
            if ($signature->mass_status) {
                $sig_severity = $this->getMassStatusSeverity($signature->mass_status);
                if ($sig_severity > $max_severity) {
                    $max_severity = $sig_severity;
                    $worst_mass_status = $signature->mass_status;
                }
            }
        }

        return $worst_mass_status;
    }

    private function getLifetimeSeverity(LifetimeStatus $status): int
    {
        return match ($status) {
            LifetimeStatus::Healthy => 1,
            LifetimeStatus::EndOfLife => 2,
            LifetimeStatus::Critical => 3,
        };
    }

    private function getNewLifetimeValue(MapConnection $mapConnection, MapConnectionData $data): LifetimeStatus
    {
        if (! $data->lifetime instanceof Optional) {
            return $data->lifetime;
        }

        $signatures = $mapConnection->signatures;
        if ($signatures->isEmpty()) {
            return $mapConnection->lifetime;
        }

        $connection_lifetime_severity = $this->getLifetimeSeverity($mapConnection->lifetime);
        $max_severity = $connection_lifetime_severity;
        $worst_lifetime = $mapConnection->lifetime;

        foreach ($signatures as $signature) {
            $signature_severity = $this->getLifetimeSeverity($signature->lifetime);
            if ($signature_severity > $max_severity) {
                $max_severity = $signature_severity;
                $worst_lifetime = $signature->lifetime;
            }
        }

        return $worst_lifetime;
    }
}
