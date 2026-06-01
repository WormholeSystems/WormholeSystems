<?php

declare(strict_types=1);

namespace App\Actions\MapWebhooks;

use App\Models\MapWebhook;

final readonly class UpdateMapWebhookAction
{
    /**
     * @param  array<string, mixed>  $data
     */
    public function handle(MapWebhook $webhook, array $data): MapWebhook
    {
        $webhook->update($data);

        return $webhook;
    }
}
