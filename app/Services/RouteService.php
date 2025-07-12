<?php

namespace App\Services;

use App\Models\Map;
use App\Models\MapConnection;
use App\Utilities\DomainLogic;
use JMGQ\AStar\AStar;
use NicolasKion\SDE\Models\SolarsystemConnection;

/**
 * @property array<int, int[]> $connections
 * @property array<int, int[]> $extra_connections
 */
class RouteService
{
    private array $connections = [];
    private array $extra_connections = [];

    function __construct()
    {
        $this->connections = SolarsystemConnection::query()
            ->select('from_solarsystem_id', 'to_solarsystem_id')
            ->get()
            ->groupBy('from_solarsystem_id')
            ->map(function ($group) {
                return $group->pluck('to_solarsystem_id')->toArray();
            })
            ->toArray();
    }


    public function find(int $from_id, int $to_id, ?Map $map = null)
    {
        if ($map) {
            $this->getExtraConnections($map);
        }

        $mergedConnections = [];

        foreach (array_keys($this->connections + $this->extra_connections) as $solarsystemId) {
            $mergedConnections[$solarsystemId] = array_merge(
                $this->connections[$solarsystemId] ?? [],
                $this->extra_connections[$solarsystemId] ?? []
            );
        }

        $domainLogic = new DomainLogic($mergedConnections);
        $star = new AStar($domainLogic);

        return $star->run($from_id, $to_id);
    }

    private function getExtraConnections(Map $map): void
    {
        $this->extra_connections = MapConnection::query()
            ->join('map_solarsystems as from', 'map_connections.from_map_solarsystem_id', '=', 'from.id')
            ->join('map_solarsystems as to', 'map_connections.to_map_solarsystem_id', '=', 'to.id')
            ->where('map_connections.map_id', $map->id)
            ->select('from.solarsystem_id as from_solarsystem_id', 'to.solarsystem_id as to_solarsystem_id')
            ->get()
            ->map(function ($connection) {
                return [
                    ['from_solarsystem_id' => $connection->from_solarsystem_id,
                        'to_solarsystem_id' => $connection->to_solarsystem_id,
                    ],
                    ['from_solarsystem_id' => $connection->to_solarsystem_id,
                        'to_solarsystem_id' => $connection->from_solarsystem_id,
                    ],
                ];
            })
            ->flatten(1)
            ->groupBy('from_solarsystem_id')
            ->map(function ($group) {
                return $group->pluck('to_solarsystem_id')->toArray();
            })
            ->toArray();
    }
}
