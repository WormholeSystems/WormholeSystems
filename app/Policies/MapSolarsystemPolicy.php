<?php

namespace App\Policies;

use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\User;

class MapSolarsystemPolicy
{
    /**
     * Create a new policy instance.
     */
    public function __construct()
    {
        //
    }

    public function create(User $user, Map $map): bool
    {
        return $user->can('update', $map);
    }

    public function update(User $user, MapSolarsystem $map_solarsystem): bool
    {
        return $user->can('update', $map_solarsystem->map);
    }

    public function delete(User $user, MapSolarsystem $map_solarsystem): bool
    {
        return $user->can('update', $map_solarsystem->map);
    }
}
