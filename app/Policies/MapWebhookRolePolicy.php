<?php

declare(strict_types=1);

namespace App\Policies;

use App\Models\Map;
use App\Models\MapWebhookRole;
use App\Models\User;

final class MapWebhookRolePolicy
{
    /**
     * Determine whether the user can create roles for the map.
     */
    public function create(User $user, Map $map): bool
    {
        return $user->can('manageAccess', $map);
    }

    /**
     * Determine whether the user can update the role.
     */
    public function update(User $user, MapWebhookRole $role): bool
    {
        return $user->can('manageAccess', $role->map);
    }

    /**
     * Determine whether the user can delete the role.
     */
    public function delete(User $user, MapWebhookRole $role): bool
    {
        return $user->can('manageAccess', $role->map);
    }
}
