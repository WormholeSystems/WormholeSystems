<?php

declare(strict_types=1);

namespace App\Policies;

use App\Models\Map;
use App\Models\MapIgnoredSolarsystem;
use App\Models\User;

final class MapIgnoredSolarsystemPolicy
{
    /**
     * Determine whether the user can create models.
     */
    public function create(User $user, Map $map): bool
    {
        return $user->can('update', $map);
    }

    /**
     * Determine whether the user can delete the model.
     */
    public function delete(User $user, MapIgnoredSolarsystem $mapIgnoredSolarsystem): bool
    {
        return $user->can('update', $mapIgnoredSolarsystem->map);
    }
}
