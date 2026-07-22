<?php

declare(strict_types=1);

namespace App\Actions\MapAlerts;

use App\Enums\MapAlertDisabledReason;
use App\Models\MapAlert;
use App\Models\User;
use App\Services\MapAlerts\MapAlertLifecycle;
use Illuminate\Support\Facades\DB;

final readonly class UpdateMapAlertAction
{
    public function __construct(private MapAlertLifecycle $lifecycle) {}

    /**
     * @param  array<string, mixed>  $data
     */
    public function handle(MapAlert $alert, array $data, User $actor): MapAlert
    {
        $isActive = array_key_exists('is_active', $data) ? (bool) $data['is_active'] : null;
        unset($data['is_active']);

        return DB::transaction(function () use ($alert, $data, $actor, $isActive): MapAlert {
            $alert->update($data);
            $this->lifecycle->updated($alert, $actor);

            if ($isActive === true) {
                $this->lifecycle->enable($alert, $actor);
            } elseif ($isActive === false) {
                $this->lifecycle->disable($alert, $actor, MapAlertDisabledReason::Manual);
            }

            return $alert->refresh();
        });
    }
}
