<?php

declare(strict_types=1);

namespace App\Scopes;

use Illuminate\Database\Eloquent\Builder;
use NicolasKion\Esi\Enums\EsiScope;

final readonly class CharacterHasRequiredScopes
{
    /**
     * @param  EsiScope[]  $scopes
     */
    public function __construct(
        private array $scopes
    ) {}

    public function __invoke(Builder $query): Builder
    {
        return $query->where(function (Builder $query): void {
            foreach ($this->scopes as $scope) {
                $query->whereHas('esiTokens.esiScopes', fn (Builder $scopeQuery) => $scopeQuery->where('name', '=', $scope->value));
            }
        });
    }
}
