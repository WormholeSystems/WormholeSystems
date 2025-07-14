<?php

namespace App\Listeners;

use App\Events\MapConnections\MapConnectionCreatedEvent;
use App\Events\MapConnections\MapConnectionDeletedEvent;
use App\Events\MapConnections\MapConnectionsDeletedEvent;
use App\Events\MapSolarsystems\MapSolarsystemCreatedEvent;
use App\Events\MapSolarsystems\MapSolarsystemDeletedEvent;
use App\Events\MapSolarsystems\MapSolarsystemsDeletedEvent;
use App\Services\RouteService;
use Illuminate\Support\Facades\Cache;

class MapRoutesBecameOutdated
{
    /**
     * Create the event listener.
     */
    public function __construct()
    {
        //
    }

    /**
     * Handle the event.
     */
    public function handle(
        MapSolarsystemCreatedEvent|
        MapSolarsystemDeletedEvent|
        MapSolarsystemsDeletedEvent|
        MapConnectionCreatedEvent|
        MapConnectionDeletedEvent|
        MapConnectionsDeletedEvent $event): void
    {
        $map_id = $event->map_id;
        $tag = sprintf(RouteService::MAP_CACHE_PATTERN, $map_id);
        Cache::tags($tag)->flush();
    }
}
