<?php

declare(strict_types=1);

namespace App\Actions\MapAlerts;

use App\Models\MapAlert;

final readonly class DeleteMapAlertAction
{
    public function handle(MapAlert $alert): void
    {
        $alert->delete();
    }
}
