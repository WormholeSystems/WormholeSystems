<?php

declare(strict_types=1);

namespace App\Actions\MapWebhooks;

use App\Models\MapWebhook;

final readonly class DeleteMapWebhookAction
{
    public function handle(MapWebhook $webhook): void
    {
        $webhook->delete();
    }
}
