<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Events\Signatures\SignatureCreatedEvent;
use App\Models\MapSolarsystem;
use App\Models\Signature;
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

            $category = $data['category'] ?? null;

            $signature = $mapSolarsystem->signatures()->create([
                'map_connection_id' => $data['map_connection_id'] ?? null,
                'signature_id' => $data['signature_id'] ?? null,
                'category' => $category,
                'type' => $data['type'] ?? null,
                'wormhole_id' => $category === 'Wormhole' ? Signature::typeToWormhole($data['type'] ?? '')?->id : null,
            ]);

            broadcast(new SignatureCreatedEvent($mapSolarsystem->map_id))->toOthers();

            return $signature;
        });
    }
}
