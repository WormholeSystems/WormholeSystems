<?php

declare(strict_types=1);

namespace App\Builders;

use App\Models\CharacterStatus;
use Illuminate\Database\Eloquent\Builder;
use NicolasKion\Esi\Enums\EsiScope;

use function assert;
use function now;

/**
 * @mixin CharacterStatus
 *
 * @template T of CharacterStatus
 *
 * @extends Builder<T>
 */
final class CharacterStatusBuilder extends Builder
{
    public function isOnline(?bool $is_online = true): self
    {
        return $this->where('is_online', $is_online);
    }

    public function hasRequiredScopes(): self
    {
        return $this->whereHas('character', function (Builder $query): CharacterBuilder {
            assert($query instanceof CharacterBuilder);

            return $query->hasTokenWithScopes([
                EsiScope::ReadOnlineStatus,
                EsiScope::ReadShip,
                EsiScope::ReadLocations,
            ]);
        });
    }

    public function doesntHaveRequiredScopes(): self
    {
        return $this->whereNot(fn (self $query): CharacterStatusBuilder => $query->hasRequiredScopes());
    }

    public function wasRecentlyActive(int $minutes_threshold = 10): self
    {
        return $this->whereHas('character.user', fn (Builder $query) => $query->where('last_active_at', '>=', now()->subMinutes($minutes_threshold)));
    }

    public function wasNotRecentlyActive(int $minutes = 10): self
    {
        return $this->whereNot(fn (self $query): CharacterStatusBuilder => $query->wasRecentlyActive($minutes));
    }
}
