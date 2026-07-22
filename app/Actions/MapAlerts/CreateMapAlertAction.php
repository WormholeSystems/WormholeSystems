<?php

declare(strict_types=1);

namespace App\Actions\MapAlerts;

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Models\MapAlert;
use App\Models\User;
use App\Services\MapAlerts\MapAlertLifecycle;
use Illuminate\Support\Facades\DB;

final readonly class CreateMapAlertAction
{
    public function __construct(private MapAlertLifecycle $lifecycle) {}

    /**
     * @param  array<string, mixed>  $data
     */
    public function handle(array $data, User $actor): MapAlert
    {
        $isActive = (bool) ($data['is_active'] ?? true);
        unset($data['is_active']);

        return DB::transaction(function () use ($data, $actor, $isActive): MapAlert {
            $alert = MapAlert::query()->create([
                ...$data,
                'created_by_user_id' => $actor->id,
                'delivery_type' => MapAlertDeliveryType::Webhook,
                'is_active' => true,
            ]);
            $this->lifecycle->created($alert, $actor);

            if (! $isActive) {
                $this->lifecycle->disable($alert, $actor, MapAlertDisabledReason::Manual);
            }

            return $alert->refresh();
        });
    }
}
