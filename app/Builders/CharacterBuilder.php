<?php

declare(strict_types=1);

namespace App\Builders;

use App\Models\Character;
use App\Models\Map;
use Illuminate\Database\Eloquent\Builder;
use NicolasKion\Esi\Enums\EsiScope;

use function array_map;

/**
 * @mixin Character
 *
 * @template T of Character
 *
 * @extends Builder<T>
 */
final class CharacterBuilder extends Builder
{
    public function hasTokenWithScopes(array $scopes): self
    {
        return $this->whereHas(
            'esiTokens',
            fn (Builder $query): array => array_map(
                fn (EsiScope $scope) => $query->whereHas(
                    'esiScopes',
                    fn (Builder $scopeQuery) => $scopeQuery->where('name', '=', $scope)
                ),
                $scopes
            )
        );
    }

    public function hasTokenWithTrackingScopes(): self
    {
        return $this->hasTokenWithScopes([
            EsiScope::ReadOnlineStatus,
            EsiScope::ReadShip,
            EsiScope::ReadLocations,
        ]);
    }

    public function hasTokenWithWaypointScopes(): self
    {
        return $this->hasTokenWithScopes([
            EsiScope::WriteWaypoint,
        ]);
    }

    public function doesntHaveTokenWithScopes(array $scopes): self
    {
        return $this->whereNot(fn (self $query): CharacterBuilder => $query->hasTokenWithScopes($scopes));
    }

    public function doesntHaveTokenWithTrackingScopes(): self
    {
        return $this->doesntHaveTokenWithScopes([
            EsiScope::ReadOnlineStatus,
            EsiScope::ReadShip,
            EsiScope::ReadLocations,
        ]);
    }

    public function shouldGetTracked(Map $map): self
    {
        return $this->whereHas('maps', fn (Builder $query) => $query->where('map_id', '=', $map->id))
            ->where('track', '=', true)
            ->where('last_tracked_at', '<', now()->subMinutes(15));
    }
}
