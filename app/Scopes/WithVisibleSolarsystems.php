<?php

declare(strict_types=1);

namespace App\Scopes;

use App\Models\Map;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Relations\Relation;

final class WithVisibleSolarsystems
{
    /**
     * @param  Builder<Map>  $query
     * @return Builder<Map>
     */
    public function __invoke(Builder $query): Builder
    {
        return $query->with([
            'mapSolarsystems' => fn (Relation $query) => $query->withCount('signatures', 'wormholeSignatures', 'mapConnections', 'uncategorizedSignatures')->with('wormholeSystem:id,threat_level', 'details'),
            'mapConnections',
        ]);
    }
}
