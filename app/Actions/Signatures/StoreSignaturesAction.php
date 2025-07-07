<?php

namespace App\Actions\Signatures;

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

        $existing = $mapSolarsystem->signatures()
            ->whereIn('signature_id', $collection->pluck('signature_id'))
            ->get()
            ->keyBy('signature_id');
    }
}
