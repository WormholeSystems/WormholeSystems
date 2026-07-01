<?php

declare(strict_types=1);

namespace App\Actions\MapWebhookRoles;

use App\Models\MapWebhookRole;

final readonly class DeleteMapWebhookRoleAction
{
    /**
     * Deleting a role detaches it from any alerts (the FK nulls out), so those alerts
     * keep firing, just without a ping.
     */
    public function handle(MapWebhookRole $role): void
    {
        $role->delete();
    }
}
