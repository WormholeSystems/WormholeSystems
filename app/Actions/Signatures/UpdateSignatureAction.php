<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Data\SignatureData;
use App\Enums\MassStatus;
use App\Events\Signatures\SignatureUpdatedEvent;
use App\Models\Signature;
use DateTimeImmutable;
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
            'marked_as_eol_at' => $eol,
            'mass_status' => $mass,
        ]);

        $signature->update([
            'marked_as_eol_at' => $eol,
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

    private function getNewEolValue(Signature $signature, SignatureData $data): ?DateTimeImmutable
    {
        if (! $data->marked_as_eol_at instanceof Optional) {
            return $data->marked_as_eol_at;
        }

        $connection_eol = $signature->mapConnection->marked_as_eol_at;
        $signature_eol = $signature->marked_as_eol_at;

        return match (true) {
            $connection_eol === null && $signature_eol !== null => $signature_eol,
            $connection_eol !== null && $signature_eol === null => $connection_eol,
            $connection_eol !== null && $signature_eol !== null => min($connection_eol, $signature_eol),
            default => null
        };
    }
}
