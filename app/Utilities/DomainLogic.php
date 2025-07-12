<?php

namespace App\Utilities;

use JMGQ\AStar\DomainLogicInterface;

class DomainLogic implements DomainLogicInterface
{
    /**
     * @param array<int, int[]> $connections
     */
    function __construct(public array $connections)
    {

    }

    public function getAdjacentNodes(mixed $node): iterable
    {
        return $this->getSolarsystemConnections($node);
    }

    public function calculateRealCost(mixed $node, mixed $adjacent): float|int
    {
        return 1;
    }

    public function calculateEstimatedCost(mixed $fromNode, mixed $toNode): float|int
    {
        return 1;
    }

    private function getSolarsystemConnections(int $solarsystemId): array
    {
        return $this->connections[$solarsystemId] ?? [];
    }
}
