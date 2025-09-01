<?php

declare(strict_types=1);

namespace App\Services;

use App\Enums\MassStatus;
use App\Models\MapConnection;
use App\Models\Signature;

final class SignatureConnectionSyncService
{
    /**
     * Sync status from signature to its connected map connection.
     */
    public function syncFromSignature(Signature $signature): void
    {
        if (! $signature->mapConnection) {
            return;
        }

        $connection = $signature->mapConnection;
        $needsUpdate = false;
        $updates = [];

        // Sync EOL status - if signature is EOL, connection should be too
        if ($signature->is_eol === true && $connection->is_eol !== true) {
            $updates['is_eol'] = true;
            $needsUpdate = true;
        }

        // Sync mass status - worse status wins
        if ($signature->mass_status && $this->isWorseMassStatus($signature->mass_status, $connection->mass_status)) {
            $updates['mass_status'] = $signature->mass_status;
            $needsUpdate = true;
        }

        if ($needsUpdate) {
            // Update without triggering events to prevent infinite loops
            $connection->updateQuietly($updates);
        }
    }

    /**
     * Sync status from map connection to all its signatures.
     */
    public function syncFromConnection(MapConnection $connection): void
    {
        $signatures = $connection->signatures;

        if ($signatures->isEmpty()) {
            return;
        }

        foreach ($signatures as $signature) {
            $needsUpdate = false;
            $updates = [];

            // Sync EOL status - if connection is EOL, signature should be too
            if ($connection->is_eol === true && $signature->is_eol !== true) {
                $updates['is_eol'] = true;
                $needsUpdate = true;
            }

            // Sync mass status - worse status wins
            if ($this->isWorseMassStatus($connection->mass_status, $signature->mass_status)) {
                $updates['mass_status'] = $connection->mass_status;
                $needsUpdate = true;
            }

            if ($needsUpdate) {
                // Update without triggering events to prevent infinite loops
                $signature->updateQuietly($updates);
            }
        }
    }

    /**
     * Sync status bidirectionally between signature and connection.
     * This method determines the "worse" values and applies them to both entities.
     */
    public function syncBidirectional(Signature $signature): void
    {
        if (! $signature->mapConnection) {
            return;
        }

        $connection = $signature->mapConnection;
        $signatureUpdates = [];
        $connectionUpdates = [];

        // Determine worse EOL status (true is worse than false/null)
        $worseEol = $signature->is_eol === true || $connection->is_eol === true;

        if ($signature->is_eol !== $worseEol) {
            $signatureUpdates['is_eol'] = $worseEol;
        }

        if ($connection->is_eol !== $worseEol) {
            $connectionUpdates['is_eol'] = $worseEol;
        }

        // Determine worse mass status
        $worseMassStatus = $this->getWorseMassStatus($signature->mass_status, $connection->mass_status);

        if ($signature->mass_status !== $worseMassStatus && $worseMassStatus instanceof MassStatus) {
            $signatureUpdates['mass_status'] = $worseMassStatus;
        }

        if ($connection->mass_status !== $worseMassStatus && $worseMassStatus instanceof MassStatus) {
            $connectionUpdates['mass_status'] = $worseMassStatus;
        }

        // Apply updates without triggering events to prevent infinite loops
        if ($signatureUpdates !== []) {
            $signature->updateQuietly($signatureUpdates);
        }

        if ($connectionUpdates !== []) {
            $connection->updateQuietly($connectionUpdates);
        }
    }

    /**
     * Determine if the first mass status is worse than the second.
     */
    private function isWorseMassStatus(?MassStatus $first, ?MassStatus $second): bool
    {
        if (! $first instanceof MassStatus) {
            return false;
        }

        if (! $second instanceof MassStatus) {
            return true;
        }

        return $this->getMassStatusSeverity($first) > $this->getMassStatusSeverity($second);
    }

    /**
     * Get the worse of two mass statuses.
     */
    private function getWorseMassStatus(?MassStatus $first, ?MassStatus $second): ?MassStatus
    {
        if (! $first instanceof MassStatus && ! $second instanceof MassStatus) {
            return null;
        }

        if (! $first instanceof MassStatus) {
            return $second;
        }

        if (! $second instanceof MassStatus) {
            return $first;
        }

        return $this->getMassStatusSeverity($first) >= $this->getMassStatusSeverity($second) ? $first : $second;
    }

    /**
     * Get numerical severity for mass status (higher = worse).
     */
    private function getMassStatusSeverity(MassStatus $status): int
    {
        return match ($status) {
            MassStatus::Fresh => 0,
            MassStatus::Unknown => 1,
            MassStatus::Reduced => 2,
            MassStatus::Critical => 3,
        };
    }
}
