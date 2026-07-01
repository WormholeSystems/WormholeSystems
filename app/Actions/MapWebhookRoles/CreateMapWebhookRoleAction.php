<?php

declare(strict_types=1);

namespace App\Actions\MapWebhookRoles;

use App\Models\MapWebhookRole;

final readonly class CreateMapWebhookRoleAction
{
    /**
     * @param  array<string, mixed>  $data
     */
    public function handle(array $data): MapWebhookRole
    {
        return MapWebhookRole::query()->create($data);
    }
}
