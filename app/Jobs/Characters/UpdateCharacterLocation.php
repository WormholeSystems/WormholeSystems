<?php

declare(strict_types=1);

namespace App\Jobs\Characters;

use App\Actions\ShipHistories\UpdateShipHistoryAction;
use App\Models\CharacterStatus;
use Illuminate\Bus\Batchable;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Queue\Middleware\WithoutOverlapping;
use Illuminate\Support\Facades\Log;
use NicolasKion\Esi\DTO\Location;
use NicolasKion\Esi\DTO\Ship;
use NicolasKion\Esi\Esi;
use Throwable;

use function assert;

final class UpdateCharacterLocation implements ShouldQueue
{
    use Batchable, Queueable;

    /**
     * Create a new job instance.
     */
    public function __construct(public int $character_status_id)
    {
        //
    }

    /**
     * Execute the job.
     *
     * @throws Throwable
     * @throws ConnectionException
     */
    public function handle(Esi $esi, UpdateShipHistoryAction $action): void
    {
        $characterStatus = CharacterStatus::query()->find($this->character_status_id);

        if ($characterStatus === null) {
            return;
        }

        $location_request = $esi->getLocation($characterStatus->character);

        if ($location_request->failed()) {
            Log::info(sprintf('Failed to fetch location for character %d', $characterStatus->character_id), (array) $location_request);

            return;
        }

        $ship_request = $esi->getShip($characterStatus->character);

        if ($ship_request->failed()) {
            Log::info(sprintf('Failed to fetch ship for character %d', $characterStatus->character_id), (array) $ship_request);

            return;
        }

        $location = $location_request->data;
        $ship = $ship_request->data;

        assert($location instanceof Location);
        assert($ship instanceof Ship);

        $characterStatus->update([
            'solarsystem_id' => $location->solar_system_id,
            'station_id' => $location->station_id,
            'structure_id' => $location->structure_id,
            'ship_name' => $ship->ship_name,
            'ship_type_id' => $ship->ship_type_id,
            'ship_item_id' => $ship->ship_item_id,
        ]);

        $action->handle(
            $characterStatus->character_id,
            $ship->ship_item_id,
            $ship->ship_type_id,
            $ship->ship_name
        );

        if ($characterStatus->wasChanged()) {
            // Mark that event should be dispatched
            $characterStatus->update(['event_queued_at' => now()]);
        }
    }

    /**
     * Get the middleware the job should pass through.
     *
     * @return array<int, object>
     */
    public function middleware(): array
    {
        return [new WithoutOverlapping((string) $this->character_status_id)->dontRelease()->expireAfter(60)];
    }
}
