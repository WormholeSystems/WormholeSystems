<?php

namespace App\Policies;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\User;

class MapPolicy
{
    /**
     * Create a new policy instance.
     */
    public function __construct()
    {
        //
    }

    public function view(User $user, Map $map): bool
    {
        return $map->mapAccessors()->whereIn('accessible_id', $user->getAccessibleIds())->exists();
    }

    public function create(User $user): bool
    {
        return true;
    }

    public function update(User $user, Map $map): bool
    {
        return $map->mapAccessors()->whereIn('accessible_id', $user->getAccessibleIds())->where('permission', Permission::Write)->exists();
    }

    public function delete(User $user, Map $map): bool
    {
        return $map->mapAccessors()->whereIn('accessible_id', $user->getAccessibleIds())->where('is_owner', true)->exists();
    }
}
