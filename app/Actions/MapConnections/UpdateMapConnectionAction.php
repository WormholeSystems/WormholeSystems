<?php

namespace App\Actions\MapConnections;

use App\Models\MapConnection;

class UpdateMapConnectionAction
{
    public function handle(MapConnection $mapConnection, array $data): MapConnection
    {
        return tap($mapConnection)->update($data);
    }
}
