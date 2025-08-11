<?php

declare(strict_types=1);

namespace App\Policies;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;

final class MapAccessPolicy
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
        return $map->mapAccessors()->whereIn('accessible_id', $user->getAccessibleIds())->where('permission', Permission::Write)->exists();
    }

    public function update(User $user, Map $map, MapAccess $access): bool
    {
        if ($access->is_owner) {
            return false;
        }

        return $map->mapAccessors()->whereIn('accessible_id', $user->getAccessibleIds())->where('permission', Permission::Write)->exists();
    }

    public function test(): bool
    {
        return false;
    }
}
