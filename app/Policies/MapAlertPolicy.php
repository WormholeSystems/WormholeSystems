<?php

declare(strict_types=1);

namespace App\Policies;

use App\Enums\MapAlertDeliveryType;
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
        return $this->canAdminister($user, $alert);
    }

    /**
     * Determine whether the user can delete the alert.
     */
    public function delete(User $user, MapAlert $alert): bool
    {
        return $this->canAdminister($user, $alert);
    }

    private function canAdminister(User $user, MapAlert $alert): bool
    {
        if ($user->can('manageAccess', $alert->map)) {
            return true;
        }

        return $alert->delivery_type === MapAlertDeliveryType::DiscordDm
            && $alert->created_by_user_id === $user->id
            && $user->can('view', $alert->map);
    }
}
