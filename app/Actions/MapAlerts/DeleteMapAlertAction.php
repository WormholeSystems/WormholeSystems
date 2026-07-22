<?php

declare(strict_types=1);

namespace App\Actions\MapAlerts;

use App\Models\MapAlert;
use App\Models\User;
use App\Services\MapAlerts\MapAlertLifecycle;

final readonly class DeleteMapAlertAction
{
    public function __construct(private MapAlertLifecycle $lifecycle) {}

    public function handle(MapAlert $alert, User $actor): void
    {
        $this->lifecycle->remove($alert, $actor);
    }
}
