<?php

declare(strict_types=1);

namespace App\Builders;

use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Builder;

/**
 * @mixin MapSolarsystem
 *
 * @template T of MapSolarsystem
 *
 * @extends Builder<T>
 */
final class MapSolarsystemBuilder extends Builder
{
    /**
     * Determine if the solarsystem has a position on the map and is therefore visible.
     */
    public function isOnMap(): self
    {
        return $this->whereNotNull('position_x')->whereNotNull('position_y');
    }

    public function isSolarsystem(Solarsystem|int $solarsystem): self
    {
        $solarsystem_id = $solarsystem instanceof Solarsystem ? $solarsystem->id : $solarsystem;

        return $this->where('solarsystem_id', $solarsystem_id);
    }
}
