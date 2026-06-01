<?php

declare(strict_types=1);

namespace App\Policies;

use App\Models\Map;
use App\Models\MapWebhook;
use App\Models\User;

final class MapWebhookPolicy
{
    /**
     * Determine whether the user can create webhooks for the map.
     */
    public function create(User $user, Map $map): bool
    {
        return $user->can('manageAccess', $map);
    }

    /**
     * Determine whether the user can update the webhook.
     */
    public function update(User $user, MapWebhook $mapWebhook): bool
    {
        return $user->can('manageAccess', $mapWebhook->map);
    }

    /**
     * Determine whether the user can delete the webhook.
     */
    public function delete(User $user, MapWebhook $mapWebhook): bool
    {
        return $user->can('manageAccess', $mapWebhook->map);
    }
}
