<?php

namespace App\Actions\Signatures;

use App\Events\Signatures\SignatureDeletedEvent;
use App\Models\Signature;
use Illuminate\Support\Facades\DB;
use Throwable;

class DeleteSignatureAction
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
    public function handle(Signature $signature): bool
    {
        return DB::transaction(function () use ($signature) {
            $signature->delete();
            broadcast(new SignatureDeletedEvent($signature->mapSolarsystem->map_id))->toOthers();

            return true;
        });
    }
}
