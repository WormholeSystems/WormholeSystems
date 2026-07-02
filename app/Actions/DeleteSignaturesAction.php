<?php

declare(strict_types=1);

namespace App\Actions;

use App\Actions\Signatures\DeleteSignatureAction;
use App\Events\Signatures\SignatureDeletedEvent;
use App\Models\MapSolarsystem;
use App\Models\Signature;
use App\Support\Broadcasting\MapBroadcaster;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteSignaturesAction
{
    public function __construct(
        private DeleteSignatureAction $action,
        private MapBroadcaster $mapBroadcaster,
    ) {}

    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(MapSolarsystem $mapSolarsystem, array $signature_ids, bool $remove_map_solarsystems = false): void
    {
        DB::transaction(function () use ($mapSolarsystem, $signature_ids, $remove_map_solarsystems): void {
            $mapSolarsystem->signatures()
                ->whereIn('signatures.id', $signature_ids)
                ->each(fn (Signature $signature): bool => $this->action->handle($signature, without_events: true, remove_map_solarsystem: $remove_map_solarsystems));

            broadcast(new SignatureDeletedEvent($mapSolarsystem->map_id))
                ->toOthers();

            $this->mapBroadcaster->signaturesChanged($mapSolarsystem);
        });
    }
}
