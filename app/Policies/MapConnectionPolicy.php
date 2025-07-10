<?php

namespace App\Policies;

use App\Models\Map;
use App\Models\MapConnection;
use App\Models\User;

class MapConnectionPolicy
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

    public function update(User $user, MapConnection $map_connection): bool
    {
        return $user->can('update', $map_connection->map);
    }

    public function delete(User $user, MapConnection $map_connection): bool
    {
        return $user->can('update', $map_connection->map);
    }
}
