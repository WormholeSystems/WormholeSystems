<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Events\Signatures\SignatureCreatedEvent;
use App\Models\MapSolarsystem;
use App\Models\Signature;
use App\Models\SignatureType;
use Illuminate\Support\Facades\DB;
use Throwable;

final class StoreSignatureAction
{
    /**
     * Create a new class instance.
     */
    public function __construct()
    {
        //
    }

    /**
     * @throws Throwable
     */
    public function handle(array $data): Signature
    {
        return DB::transaction(function () use ($data) {
            $mapSolarsystem = MapSolarsystem::query()->findOrFail($data['map_solarsystem_id']);

            // Get wormhole_id if this is a wormhole signature type
            $wormholeId = null;
            if (isset($data['signature_type_id'])) {
                $signatureType = SignatureType::query()->find($data['signature_type_id']);
                $wormholeId = $signatureType?->wormhole?->id;
            }

            $signature = $mapSolarsystem->signatures()->create([
                'map_connection_id' => $data['map_connection_id'] ?? null,
                'signature_id' => $data['signature_id'] ?? null,
                'signature_category_id' => $data['signature_category_id'] ?? null,
                'signature_type_id' => $data['signature_type_id'] ?? null,
                'wormhole_id' => $wormholeId,
            ]);

            broadcast(new SignatureCreatedEvent($mapSolarsystem->map_id))->toOthers();

            return $signature;
        });
    }
}
