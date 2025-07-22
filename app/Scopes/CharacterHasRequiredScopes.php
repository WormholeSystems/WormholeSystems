<?php

namespace App\Scopes;

use App\Models\Character;
use Illuminate\Database\Eloquent\Builder;
use NicolasKion\Esi\Enums\EsiScope;

readonly class CharacterHasRequiredScopes
{
    /**
     * @param  EsiScope[]  $scopes
     */
    public function __construct(
        private array $scopes
    ) {}

    /**
     * @param  Builder<Character>  $query
     * @return Builder<Character>
     */
    public function __invoke(Builder $query): Builder
    {
        return $query->where(function (Builder $query): void {
            foreach ($this->scopes as $scope) {
                $query->whereHas('esiTokens.esiScopes', fn (Builder $scopeQuery) => $scopeQuery->where('name', '=', $scope->value));
            }
        });
    }
}
