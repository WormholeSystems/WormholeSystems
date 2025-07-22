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
        return DB::transaction(function () use ($signature, $data): \App\Models\Signature {
            $signature->update($data);

            broadcast(new SignatureUpdatedEvent($signature->mapSolarsystem->map_id))->toOthers();

            return $signature;
        });
    }
}
