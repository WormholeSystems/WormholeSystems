<?php

namespace App\Policies;

use App\Models\Map;
use App\Models\MapRouteSolarsystem;
use App\Models\User;

class MapRouteSolarsystemPolicy
{
    public function create(User $user, Map $map): bool
    {
        return $user->can('update', $map);
    }

    /**
     * Determine whether the user can update the model.
     */
    public function update(User $user, MapRouteSolarsystem $mapRouteSolarsystem): bool
    {
        return $user->can('update', $mapRouteSolarsystem->map);
    }

    /**
     * Determine whether the user can delete the model.
     */
    public function delete(User $user, MapRouteSolarsystem $mapRouteSolarsystem): bool
    {
        return $user->can('update', $mapRouteSolarsystem->map);
    }
}
