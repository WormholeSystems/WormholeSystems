<?php

declare(strict_types=1);

namespace App\Policies;

use App\Models\MapConnection;
use App\Models\MapConnectionJump;
use App\Models\User;

final class MapConnectionJumpPolicy
{
    public function create(User $user, MapConnection $map_connection): bool
    {
        return $user->can('update', $map_connection);
    }

    public function update(User $user, MapConnectionJump $map_connection_jump): bool
    {
        return $map_connection_jump->mapConnection instanceof MapConnection
            && $user->can('update', $map_connection_jump->mapConnection);
    }

    public function delete(User $user, MapConnectionJump $map_connection_jump): bool
    {
        return $this->update($user, $map_connection_jump);
    }
}
