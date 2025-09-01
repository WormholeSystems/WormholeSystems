<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Data\MapConnectionData;
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

            $this->syncMassAndEol($mapConnection, $data);

            broadcast(new MapConnectionUpdatedEvent($mapConnection->map_id))->toOthers();

            return $mapConnection;
        });
    }

    private function syncMassAndEol(MapConnection $mapConnection, MapConnectionData $data): void
    {
        $signatures = $mapConnection->signatures;

        if ($signatures->isEmpty()) {
            return;
        }

        $eol = $this->getNewEolValue($mapConnection, $data);
        $mass = $this->getNewMassStatus($mapConnection, $data);

        $mapConnection->update([
            'is_eol' => $eol,
            'mass_status' => $mass,
        ]);

        foreach ($signatures as $signature) {
            $signature->update([
                'is_eol' => $eol,
                'mass_status' => $mass,
            ]);
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

    private function getNewEolValue(MapConnection $mapConnection, MapConnectionData $data): bool
    {
        if (! $data->is_eol instanceof Optional) {
            return $data->is_eol;
        }

        $signatures = $mapConnection->signatures;
        if ($signatures->isEmpty()) {
            return $mapConnection->is_eol ?? false;
        }

        $is_eol = $mapConnection->is_eol ?? false;

        foreach ($signatures as $signature) {
            if ($signature->is_eol) {
                $is_eol = true;
                break;
            }
        }

        return $is_eol;
    }
}
