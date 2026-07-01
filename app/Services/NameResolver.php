<?php

declare(strict_types=1);

namespace App\Services;

interface NameResolver
{
    /**
     * Resolve EVE entity ids to their names. Ids that can't be resolved are omitted.
     *
     * @param  int[]  $ids
     * @return array<int, string>
     */
    public function resolve(array $ids): array;
}
