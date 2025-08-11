<?php

declare(strict_types=1);

namespace App\Scopes;

use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use Illuminate\Database\Eloquent\Builder;

final readonly class CharacterHasMapAccess
{
    public function __construct(private Map $map) {}

    /**
     * @param  Builder<Character>  $query
     * @return Builder<Character>
     */
    public function __invoke(Builder $query): Builder
    {
        return $query
            ->whereExists(MapAccess::query()
                ->whereBelongsTo($this->map)
                ->where(fn (Builder $query) => $query
                    ->whereColumn('accessible_id', 'characters.id')
                    ->orWhereColumn('accessible_id', 'characters.corporation_id')
                    ->orWhereColumn('accessible_id', 'characters.alliance_id'
                    )
                )
            );
    }
}
