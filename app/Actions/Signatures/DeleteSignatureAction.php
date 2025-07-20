<?php

namespace App\Actions\Signatures;

use App\Actions\MapConnections\DeleteMapConnectionAction;
use App\Events\Signatures\SignatureDeletedEvent;
use App\Models\Signature;
use Illuminate\Support\Facades\DB;
use Throwable;

readonly class DeleteSignatureAction
{
    /**
     * Create a new class instance.
     */
    public function __construct(
        private DeleteMapConnectionAction $deleteMapConnectionAction,
    ) {
        //
    }

    /**
     * @throws Throwable
     */
    public function handle(Signature $signature): bool
    {
        return DB::transaction(function () use ($signature) {
            broadcast(new SignatureDeletedEvent($signature->mapSolarsystem->map_id))->toOthers();
            $this->deleteMapConnection($signature);
            $signature->delete();

            return true;
        });
    }

    private function deleteMapConnection(Signature $signature): void
    {
        if ($signature->map_connection_id) {
            $this->deleteMapConnectionAction->handle($signature->mapConnection);
        }
    }
}
