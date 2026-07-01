<?php

declare(strict_types=1);

namespace App\Policies;

use App\Models\Map;
use App\Models\MapAlert;
use App\Models\User;

final class MapAlertPolicy
{
    /**
     * Determine whether the user can create alerts for the map.
     */
    public function create(User $user, Map $map): bool
    {
        return $user->can('manageAccess', $map);
    }

    /**
     * Determine whether the user can update the alert.
     */
    public function update(User $user, MapAlert $alert): bool
    {
        return $user->can('manageAccess', $alert->map);
    }

    /**
     * Determine whether the user can delete the alert.
     */
    public function delete(User $user, MapAlert $alert): bool
    {
        return $user->can('manageAccess', $alert->map);
    }
}
