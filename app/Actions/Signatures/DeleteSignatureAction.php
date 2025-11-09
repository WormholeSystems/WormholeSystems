<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Actions\MapConnections\DeleteMapConnectionAction;
use App\Events\Signatures\SignatureDeletedEvent;
use App\Models\Signature;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteSignatureAction
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
    public function handle(Signature $signature, bool $without_events = false, bool $remove_map_solarsystem = false): bool
    {
        return DB::transaction(function () use ($signature, $without_events, $remove_map_solarsystem): true {
            broadcast_unless($without_events, new SignatureDeletedEvent($signature->mapSolarsystem->map_id))->toOthers();
            $this->deleteMapConnection($signature, $remove_map_solarsystem);
            $signature->delete();

            return true;
        });
    }

    /**
     * @throws Throwable
     */
    private function deleteMapConnection(Signature $signature, bool $remove_map_solarsystem = false): void
    {
        if ($signature->map_connection_id) {
            if ($signature->mapConnection->signatures()->count() > 1) {
                return;
            }
            $this->deleteMapConnectionAction->handle($signature->mapConnection, $remove_map_solarsystem);
        }
    }
}
