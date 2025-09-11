<?php

declare(strict_types=1);

namespace App\Builders;

use App\Enums\LifetimeStatus;
use App\Models\CharacterStatus;
use App\Models\MapConnection;
use Illuminate\Database\Eloquent\Builder;

/**
 * @mixin CharacterStatus
 *
 * @template T of MapConnection
 *
 * @extends Builder<T>
 */
final class MapConnectionBuilder extends Builder
{
    public function connectsToWormhole(): self
    {
        return $this->where(
            fn (Builder $query) => $query
                ->whereHas('fromMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem'))
                ->orWhereHas('toMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem'))
        );
    }

    public function isNotTimeCritical(): self
    {
        return $this->where('lifetime', '!=', LifetimeStatus::Critical);
    }
}
