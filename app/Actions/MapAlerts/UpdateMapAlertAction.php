<?php

declare(strict_types=1);

namespace App\Actions\MapAlerts;

use App\Models\MapAlert;

final readonly class UpdateMapAlertAction
{
    /**
     * @param  array<string, mixed>  $data
     */
    public function handle(MapAlert $alert, array $data): MapAlert
    {
        $alert->update($data);

        return $alert;
    }
}
