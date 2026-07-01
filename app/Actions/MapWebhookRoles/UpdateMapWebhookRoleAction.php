<?php

declare(strict_types=1);

namespace App\Actions\MapWebhookRoles;

use App\Models\MapWebhookRole;

final readonly class UpdateMapWebhookRoleAction
{
    /**
     * @param  array<string, mixed>  $data
     */
    public function handle(MapWebhookRole $role, array $data): MapWebhookRole
    {
        $role->update($data);

        return $role;
    }
}
