<?php

declare(strict_types=1);

namespace App\Actions\MapWebhooks;

use App\Models\MapWebhook;

final readonly class CreateMapWebhookAction
{
    /**
     * @param  array<string, mixed>  $data
     */
    public function handle(array $data): MapWebhook
    {
        return MapWebhook::query()->create($data);
    }
}
