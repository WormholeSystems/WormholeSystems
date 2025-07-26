<?php

declare(strict_types=1);

namespace App\Actions;

use App\Actions\Signatures\DeleteSignatureAction;
use App\Events\MapConnections\MapConnectionsDeletedEvent;
use App\Events\Signatures\SignatureDeletedEvent;
use App\Models\MapSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteSignaturesAction
{
    public function __construct(private DeleteSignatureAction $action) {}

    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(MapSolarsystem $mapSolarsystem, array $signature_ids): void
    {
        DB::transaction(function () use ($mapSolarsystem, $signature_ids): void {
            $mapSolarsystem->signatures()
                ->whereIn('signatures.id', $signature_ids)
                ->each(fn ($signature): bool => $this->action->handle($signature, without_events: true));

            broadcast(new MapConnectionsDeletedEvent(
                map_id: $mapSolarsystem->map_id,
            ))->toOthers();
            broadcast(new SignatureDeletedEvent($mapSolarsystem->map_id))
                ->toOthers();
        });
    }
}
