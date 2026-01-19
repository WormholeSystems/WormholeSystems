<?php

declare(strict_types=1);

namespace App\Builders;

use App\Models\EsiToken;
use Illuminate\Database\Eloquent\Builder;
use NicolasKion\Esi\Enums\EsiScope;

use function array_map;

/**
 * @mixin EsiToken
 *
 * @template T of EsiToken
 *
 * @extends Builder<T>
 */
final class EsiTokenBuilder extends Builder
{
    public function hasScopes(array $scopes): self
    {
        return $this->whereHas(
            'esiScopes',
            fn (Builder $query): array => array_map(
                fn (EsiScope $scope) => $query->where('name', '=', $scope),
                $scopes
            )
        );
    }

    public function hasWaypointScopes(): self
    {
        return $this->hasScopes([
            EsiScope::WriteWaypoint,
        ]);
    }
}
