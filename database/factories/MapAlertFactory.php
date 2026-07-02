<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Enums\JumpShipType;
use App\Enums\MapWebhookType;
use App\Models\MapAlert;
use App\Models\MapWebhook;
use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapAlert>
 */
final class MapAlertFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'map_webhook_id' => MapWebhook::factory(),
            'map_id' => fn (array $attributes): int => MapWebhook::query()->whereKey($attributes['map_webhook_id'])->value('map_id'),
            'map_webhook_role_id' => null,
            'type' => MapWebhookType::Proximity,
            'target_solarsystem_id' => fn () => Solarsystem::query()->inRandomOrder()->first()->id,
            'max_jumps' => fake()->numberBetween(1, 15),
            'is_active' => true,
        ];
    }

    public function inactive(): self
    {
        return $this->state(fn (): array => ['is_active' => false]);
    }

    public function jumpRange(JumpShipType $ship = JumpShipType::Dreadnought, int $jdcLevel = 5, bool $includeHighsec = false): self
    {
        return $this->state(fn (): array => [
            'type' => MapWebhookType::JumpRange,
            'ship_type' => $ship,
            'jdc_level' => $jdcLevel,
            'include_highsec' => $includeHighsec,
            'max_jumps' => null,
        ]);
    }

    /**
     * @param  array<int, array{subject: string, side: string, mode: string, ids: int[]}>  $filters
     */
    public function killmail(array $filters = []): self
    {
        return $this->state(fn (): array => [
            'type' => MapWebhookType::Killmail,
            'target_solarsystem_id' => null,
            'filters' => $filters,
        ]);
    }
}
