<?php

declare(strict_types=1);

namespace App\Policies;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\User;

final class MapPolicy
{
    /**
     * Create a new policy instance.
     */
    public function __construct()
    {
        //
    }

    public function viewAny(): bool
    {
        return true;
        // Allow authenticated users to view maps list
    }

    public function view(User $user, Map $map): bool
    {
        return $map->mapAccessors()->whereIn('accessible_id', $user->getAccessibleIds())->exists();
    }

    public function create(): bool
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
