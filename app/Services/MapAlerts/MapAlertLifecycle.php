<?php

declare(strict_types=1);

namespace App\Services\MapAlerts;

use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertEventAction;
use App\Models\MapAlert;
use App\Models\MapAlertEvent;
use App\Models\User;
use Illuminate\Support\Facades\DB;

final class MapAlertLifecycle
{
    public function created(MapAlert $alert, User $actor): void
    {
        DB::transaction(function () use ($alert, $actor): void {
            $this->lock($alert);
            $this->record($alert, $actor, MapAlertEventAction::Created);
        });
    }

    public function updated(MapAlert $alert, User $actor): void
    {
        DB::transaction(function () use ($alert, $actor): void {
            $this->lock($alert);
            $this->record($alert, $actor, MapAlertEventAction::Updated);
        });
    }

    public function enable(MapAlert $alert, User $actor): void
    {
        DB::transaction(function () use ($alert, $actor): void {
            $this->lock($alert);

            if ($alert->is_active
                && $alert->disabled_at === null
                && $alert->disabled_by_user_id === null
                && $alert->disabled_reason === null) {
                return;
            }

            $alert->update([
                'is_active' => true,
                'disabled_at' => null,
                'disabled_by_user_id' => null,
                'disabled_reason' => null,
            ]);
            $this->record($alert, $actor, MapAlertEventAction::Enabled);
        });
    }

    public function disable(MapAlert $alert, ?User $actor, MapAlertDisabledReason $reason, ?string $detail = null): void
    {
        DB::transaction(function () use ($alert, $actor, $reason, $detail): void {
            $this->lock($alert);

            if (! $alert->is_active
                && $alert->disabled_reason === $reason
                && $alert->disabled_by_user_id === $actor?->id) {
                return;
            }

            $alert->update([
                'is_active' => false,
                'disabled_at' => now(),
                'disabled_by_user_id' => $actor?->id,
                'disabled_reason' => $reason,
            ]);
            $this->record($alert, $actor, MapAlertEventAction::Disabled, $detail);
        });
    }

    public function remove(MapAlert $alert, User $actor, ?string $reason = null): void
    {
        DB::transaction(function () use ($alert, $actor, $reason): void {
            $this->lock($alert);
            $alert->update(['deleted_by_user_id' => $actor->id]);
            $this->record($alert, $actor, MapAlertEventAction::Removed, $reason);
            $alert->delete();
        });
    }

    private function record(MapAlert $alert, ?User $actor, MapAlertEventAction $action, ?string $reason = null): void
    {
        MapAlertEvent::query()->create([
            'map_alert_id' => $alert->id,
            'map_id' => $alert->map_id,
            'actor_user_id' => $actor?->id,
            'actor_name' => $actor?->alertDisplayName(),
            'action' => $action,
            'snapshot' => $alert->attributesToArray(),
            'reason' => $reason,
        ]);
    }

    private function lock(MapAlert $alert): void
    {
        $locked = MapAlert::query()->whereKey($alert->getKey())->lockForUpdate()->firstOrFail();
        $alert->setRawAttributes($locked->getRawOriginal(), true);
    }
}
