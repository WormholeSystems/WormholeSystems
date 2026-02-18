<?php

declare(strict_types=1);

namespace App\Policies;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\User;

final class MapPolicy
{
    public function viewAny(): bool
    {
        return true;
    }

    public function view(?User $user, Map $map): bool
    {
        if ($map->isPubliclyAccessible()) {
            return true;
        }

        if (! $user instanceof User) {
            return false;
        }

        return $map->mapAccessors()->notExpired()->whereIn('accessible_id', $user->getAccessibleIds())->exists();
    }

    public function viewCharacters(?User $user, Map $map): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        $permission = $map->getUserPermission($user);

        return $permission instanceof Permission && $permission->isAtLeast(Permission::Member);
    }

    public function create(): bool
    {
        return true;
    }

    public function update(?User $user, Map $map): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        $permission = $map->getUserPermission($user);

        return $permission instanceof Permission && $permission->isAtLeast(Permission::Member);
    }

    public function manageAccess(?User $user, Map $map): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        $permission = $map->getUserPermission($user);

        return $permission instanceof Permission && $permission->isAtLeast(Permission::Manager);
    }

    public function delete(?User $user, Map $map): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        return $map->mapAccessors()
            ->notExpired()
            ->whereIn('accessible_id', $user->getAccessibleIds())
            ->where('is_owner', true)
            ->exists();
    }
}
