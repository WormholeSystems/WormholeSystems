<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Enums\MapAlertEventAction;
use App\Models\MapAlert;
use App\Models\MapAlertEvent;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapAlertEvent>
 */
final class MapAlertEventFactory extends Factory
{
    /** @return array<string, mixed> */
    public function definition(): array
    {
        return [
            'map_alert_id' => MapAlert::factory(),
            'map_id' => fn (array $attributes): int => MapAlert::query()->whereKey($attributes['map_alert_id'])->value('map_id'),
            'actor_user_id' => User::factory(),
            'actor_name' => fn (array $attributes): ?string => User::query()->find($attributes['actor_user_id'])?->alertDisplayName(),
            'action' => MapAlertEventAction::Created,
            'snapshot' => [],
            'reason' => null,
        ];
    }
}
