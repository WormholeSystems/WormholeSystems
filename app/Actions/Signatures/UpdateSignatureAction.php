<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Data\SignatureData;
use App\Enums\MassStatus;
use App\Events\Signatures\SignatureUpdatedEvent;
use App\Models\Signature;
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

            $signature->update($data->toArray());

            if ($signature->category === 'Wormhole') {
                $signature->wormhole_id = Signature::typeToWormhole($signature->type)?->id;
                $signature->save();
            }

            $this->syncMassAndEol($signature, $data);

            broadcast(new SignatureUpdatedEvent($signature->mapSolarsystem->map_id))->toOthers();

            return $signature;
        });
    }

    private function syncMassAndEol(Signature $signature, SignatureData $data): void
    {
        if ($signature->mapConnection === null) {
            return;
        }

        $eol = $this->getNewEolValue($signature, $data);
        $mass = $this->getNewMassStatus($signature, $data);

        $signature->mapConnection->update([
            'is_eol' => $eol,
            'mass_status' => $mass,
        ]);

        $signature->update([
            'is_eol' => $eol,
            'mass_status' => $mass,
        ]);
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

    private function getNewEolValue(Signature $signature, SignatureData $data): bool
    {
        if (! $data->is_eol instanceof Optional) {
            return $data->is_eol;
        }

        return $signature->is_eol || $signature->mapConnection->is_eol;
    }
}
