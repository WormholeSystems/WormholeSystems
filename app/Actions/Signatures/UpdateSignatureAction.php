<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Events\Signatures\SignatureUpdatedEvent;
use App\Models\Signature;
use Illuminate\Support\Facades\DB;
use Throwable;

final class UpdateSignatureAction
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
        return DB::transaction(function () use ($signature, $data): Signature {
            $signature->update($data);

            if ($signature->category === 'Wormhole') {
                $signature->wormhole_id = Signature::typeToWormhole($signature->type)?->id;
                $signature->save();
            }

            broadcast(new SignatureUpdatedEvent($signature->mapSolarsystem->map_id))->toOthers();

            return $signature;
        });
    }
}
