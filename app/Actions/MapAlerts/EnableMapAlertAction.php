<?php

declare(strict_types=1);

namespace App\Actions\MapAlerts;

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Models\MapAlert;
use App\Models\User;
use App\Services\MapAlerts\MapAlertLifecycle;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Gate;

final readonly class EnableMapAlertAction
{
    public function __construct(private MapAlertLifecycle $lifecycle) {}

    public function handle(MapAlert $alert, User $actor): ?string
    {
        return DB::transaction(function () use ($alert, $actor): ?string {
            $alert = MapAlert::query()
                ->with(['creator', 'map'])
                ->lockForUpdate()
                ->findOrFail($alert->id);

            if (Gate::forUser($actor)->denies('update', $alert)) {
                return 'That alert does not exist.';
            }

            $isManager = Gate::forUser($actor)->allows('manageAccess', $alert->map);
            if (! $isManager && $alert->disabled_reason === MapAlertDisabledReason::Manual && $alert->disabled_by_user_id !== null && $alert->disabled_by_user_id !== $actor->id) {
                return 'A map manager disabled this alert. A manager must re-enable it.';
            }

            if ($alert->delivery_type === MapAlertDeliveryType::DiscordDm
                && ($alert->creator === null || Gate::forUser($alert->creator)->denies('view', $alert->map))) {
                return 'The alert creator needs map access before this alert can be enabled.';
            }

            if ($alert->requiresCreatorDiscordAccount()
                && ($alert->creator === null || $alert->creator->discordAccount()->doesntExist())) {
                return 'The alert creator must link Discord before this alert can be enabled.';
            }

            $this->lifecycle->enable($alert, $actor);

            return null;
        });
    }
}
