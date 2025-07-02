<?php

namespace App\Actions\MapConnections;

use App\Models\MapConnection;

class DeleteMapConnectionAction
{
    public function handle(MapConnection $mapConnection): void
    {
        $mapConnection->delete();
    }
}
