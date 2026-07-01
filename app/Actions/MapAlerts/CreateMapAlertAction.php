<?php

declare(strict_types=1);

namespace App\Actions\MapAlerts;

use App\Models\MapAlert;

final readonly class CreateMapAlertAction
{
    /**
     * @param  array<string, mixed>  $data
     */
    public function handle(array $data): MapAlert
    {
        return MapAlert::query()->create($data);
    }
}
