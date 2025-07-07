<?php

namespace App\Actions\Map;

use App\Events\Maps\MapUpdatedEvent;
use App\Models\Map;

class UpdateMapAction
{
    /**
     * Create a new class instance.
     */
    public function __construct()
    {
        //
    }

    public function handle(Map $map, array $data): Map
    {
        $map->update($data);

        broadcast(new MapUpdatedEvent($map))->toOthers();

        return $map;
    }
}
