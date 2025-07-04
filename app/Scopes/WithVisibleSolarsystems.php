<?php

namespace App\Scopes;

use App\Models\Map;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Relations\HasMany;

class WithVisibleSolarsystems
{
    /**
     * @param  Builder<Map>  $query
     * @return Builder<Map>
     */
    public function __invoke(Builder $query): Builder
    {
        return $query->with([
            'mapSolarsystems' => fn (HasMany $query) => $query->whereNotNull('position_x'),
            'mapConnections' => fn (HasMany $query) => $query->whereDoesntHave('fromMapSolarsystem', fn (Builder $query) => $query->whereNull('position_x'))
                ->whereDoesntHave('toMapSolarsystem', fn (Builder $query) => $query->whereNull('position_x')),
        ]);
    }
}
