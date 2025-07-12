<?php

declare(strict_types=1);

namespace App\Actions;

use App\Models\Character;
use Illuminate\Http\Client\ConnectionException;
use NicolasKion\Esi\Esi;

final readonly class SetWaypointAction
{
    public function __construct(private Esi $esi) {}

    /**
     * Execute the action.
     *
     * @throws ConnectionException
     */
    public function handle(Character $character, array $data): void
    {
        $this->esi->setWaypoint($character,
            destination_id: $data['destination_id'],
            add_to_beginning: $data['add_to_beginning'],
            clear_other_waypoints: $data['clear_other_waypoints'],
        );
    }
}
