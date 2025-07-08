<?php

namespace App\Actions\Signatures;

use App\Events\MapSolarsystems\MapSolarsystemCreatedEvent;
use App\Models\MapSolarsystem;

class StoreSignaturesAction
{
    /**
     * Create a new class instance.
     */
    public function __construct()
    {
        //
    }

    public function handle(MapSolarsystem $mapSolarsystem, array $signatures): void
    {
        $collection = collect($signatures);

        $existing = $mapSolarsystem->signatures;

        $missing = $existing->pluck('signature_id')
            ->diff($collection->pluck('signature_id'));

        $collection->each(function (array $signature) use ($mapSolarsystem, $existing) {
            $signatureModel = $existing->firstWhere('signature_id', $signature['signature_id']);

            if ($signatureModel) {
                $this->updateSignature($signatureModel, $signature);

                return;
            }

            $this->createSignature($mapSolarsystem, $signature);
        });

        $mapSolarsystem->signatures()
            ->whereIn('signature_id', $missing)
            ->delete();

        broadcast(new MapSolarsystemCreatedEvent($mapSolarsystem->map_id))
            ->toOthers();
    }

    protected function createSignature(MapSolarsystem $mapSolarsystem, array $signature): void
    {
        $mapSolarsystem->signatures()->create([
            'signature_id' => $signature['signature_id'],
            'type' => $signature['type'],
            'category' => $signature['category'] ?? null,
            'name' => $signature['name'] ?? null,
        ]);
    }

    protected function updateSignature($signatureModel, array $signature): void
    {
        $signatureModel->update([
            'type' => $signature['type'] ?? $signatureModel->type,
            'category' => $signature['category'] ?? $signatureModel->category,
            'name' => $signature['name'] ?? $signatureModel->name,
        ]);
    }
}
