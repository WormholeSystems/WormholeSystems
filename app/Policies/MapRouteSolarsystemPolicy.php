<?php

declare(strict_types=1);

namespace App\Policies;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\MapRouteSolarsystem;
use App\Models\User;

/**
 * Shared watchlist rows are map configuration (managers only); personal rows belong to
 * their owner, as long as the owner still has an accessor row on the map.
 */
final class MapRouteSolarsystemPolicy
{
    public function create(User $user, Map $map, bool $shared = false): bool
    {
        if ($shared) {
            return $user->can('updateSettings', $map);
        }

        return $map->getUserPermission($user) instanceof Permission;
    }

    /**
     * Determine whether the user can update the model.
     */
    public function update(User $user, MapRouteSolarsystem $mapRouteSolarsystem): bool
    {
        return $this->canEdit($user, $mapRouteSolarsystem);
    }

    /**
     * Determine whether the user can delete the model.
     */
    public function delete(User $user, MapRouteSolarsystem $mapRouteSolarsystem): bool
    {
        return $this->canEdit($user, $mapRouteSolarsystem);
    }

    private function canEdit(User $user, MapRouteSolarsystem $mapRouteSolarsystem): bool
    {
        if ($mapRouteSolarsystem->isShared()) {
            return $user->can('updateSettings', $mapRouteSolarsystem->map);
        }

        return $mapRouteSolarsystem->user_id === $user->id
            && $mapRouteSolarsystem->map->getUserPermission($user) instanceof Permission;
    }
}
