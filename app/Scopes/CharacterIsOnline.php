<?php

namespace App\Scopes;

use App\Models\Character;
use Illuminate\Database\Eloquent\Builder;

readonly class CharacterIsOnline
{
    /**
     * @param  Builder<Character>  $query
     * @return Builder<Character>
     */
    public function __invoke(Builder $query): Builder
    {
        return $query
            ->whereRelation('characterStatus', 'is_online', true);
    }
}
