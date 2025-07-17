<?php

namespace App\Actions\Signatures;

use App\Events\Signatures\SignatureUpdatedEvent;
use App\Models\Signature;
use Illuminate\Support\Facades\DB;
use Throwable;

class UpdateSignatureAction
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
    public function handle(Signature $signature, array $data): Signature
    {
        return DB::transaction(function () use ($signature, $data) {
            $signature->update([
                'map_connection_id' => $data['map_connection_id'] ?? null,
                'signature_id' => $data['signature_id'] ?? null,
                'category' => $data['category'] ?? null,
                'type' => $data['type'] ?? null,
            ]);

            broadcast(new SignatureUpdatedEvent($signature->mapSolarsystem->map_id))->toOthers();

            return $signature;
        });
    }
}
