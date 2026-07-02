<?php

declare(strict_types=1);

namespace App\Actions\Map;

use App\Models\Map;
use App\Support\Broadcasting\MapBroadcaster;

final class UpdateMapAction
{
    public function __construct(private readonly MapBroadcaster $mapBroadcaster) {}

    public function handle(Map $map, array $data): Map
    {
        $map->update($data);

        $this->mapBroadcaster->metadataUpdated($map);

        return $map;
    }
}
