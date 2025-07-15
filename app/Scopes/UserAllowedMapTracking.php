<?php

namespace App\Scopes;

use App\Models\Character;
use App\Models\Map;
use Illuminate\Database\Eloquent\Builder;

readonly class UserAllowedMapTracking
{
    public function __construct(private Map $map) {}

    /**
     * @param  Builder<Character>  $query
     * @return Builder<Character>
     */
    public function __invoke(Builder $query): Builder
    {
        return $query
            ->whereHas(
                'user.mapUserSettings',
                fn (Builder $builder) => $builder
                    ->whereBelongsTo($this->map)
                    ->where('tracking_allowed', true)
            );
    }
}
