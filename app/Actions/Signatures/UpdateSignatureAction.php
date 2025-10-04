<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Data\SignatureData;
use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Events\Signatures\SignatureUpdatedEvent;
use App\Models\Signature;
use App\Models\SignatureType;
use Illuminate\Support\Facades\DB;
use Spatie\LaravelData\Optional;
use Throwable;

final class UpdateSignatureAction
{
    /**
     * @throws Throwable
     */
    public function handle(Signature $signature, SignatureData $data): Signature
    {
        return DB::transaction(function () use ($signature, $data): Signature {
            $updateData = $data->toArray();

            // Update wormhole_id if signature_type_id changed
            if (! $data->signature_type_id instanceof Optional && $data->signature_type_id) {
                $signatureType = SignatureType::query()->find($data->signature_type_id);
                $updateData['wormhole_id'] = $signatureType?->wormhole?->id;
            }

            $signature->update($updateData);

            $this->syncMassAndLifetime($signature, $data);

            broadcast(new SignatureUpdatedEvent($signature->mapSolarsystem->map_id))->toOthers();

            return $signature;
        });
    }

    private function syncMassAndLifetime(Signature $signature, SignatureData $data): void
    {
        if ($signature->mapConnection === null) {
            return;
        }

        $lifetime = $this->getNewLifetimeValue($signature, $data);
        $mass = $this->getNewMassStatus($signature, $data);

        $connectionUpdateData = ['mass_status' => $mass];
        $signatureUpdateData = ['mass_status' => $mass];

        // Only update lifetime and timestamp if lifetime actually changed
        if ($signature->mapConnection->lifetime !== $lifetime) {
            $connectionUpdateData['lifetime'] = $lifetime;
            $connectionUpdateData['lifetime_updated_at'] = now();
        }

        if ($signature->lifetime !== $lifetime) {
            $signatureUpdateData['lifetime'] = $lifetime;
            $signatureUpdateData['lifetime_updated_at'] = now();
        }

        $signature->mapConnection->update($connectionUpdateData);
        $signature->update($signatureUpdateData);
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

    private function getNewMassStatus(Signature $signature, SignatureData $data): ?MassStatus
    {
        if (! $data->mass_status instanceof Optional) {
            return $signature->mass_status;
        }

        $sig_mass_severity = $signature->mass_status ? $this->getMassStatusSeverity($signature->mass_status) : 0;
        $conn_mass_severity = $signature->mapConnection->mass_status ? $this->getMassStatusSeverity($signature->mapConnection->mass_status) : 0;

        return match (true) {
            $sig_mass_severity <= $conn_mass_severity => $signature->mapConnection->mass_status,
            default => $signature->mass_status,
        };
    }

    private function getLifetimeSeverity(LifetimeStatus $status): int
    {
        return match ($status) {
            LifetimeStatus::Healthy => 1,
            LifetimeStatus::EndOfLife => 2,
            LifetimeStatus::Critical => 3,
        };
    }

    private function getNewLifetimeValue(Signature $signature, SignatureData $data): LifetimeStatus
    {
        if (! $data->lifetime instanceof Optional) {
            return $data->lifetime;
        }

        $connection_lifetime_severity = $this->getLifetimeSeverity($signature->mapConnection->lifetime);
        $signature_lifetime_severity = $this->getLifetimeSeverity($signature->lifetime);

        return match (true) {
            $signature_lifetime_severity >= $connection_lifetime_severity => $signature->lifetime,
            default => $signature->mapConnection->lifetime,
        };
    }
}
