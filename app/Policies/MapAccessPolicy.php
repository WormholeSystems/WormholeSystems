<?php

declare(strict_types=1);

namespace App\Policies;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;

final class MapAccessPolicy
{
    public function create(User $user, Map $map): bool
    {
        $permission = $map->getUserPermission($user);

        return $permission instanceof Permission && $permission->isAtLeast(Permission::Manager);
    }

    public function update(User $user, Map $map, MapAccess $access): bool
    {
        if ($access->is_owner) {
            return false;
        }

        $permission = $map->getUserPermission($user);

        return $permission instanceof Permission && $permission->isAtLeast(Permission::Manager);
    }

    public function test(): bool
    {
        return false;
    }
}
