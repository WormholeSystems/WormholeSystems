<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Enums\MapWebhookType;
use App\Models\Map;
use App\Models\MapWebhook;
use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapWebhook>
 */
final class MapWebhookFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'map_id' => Map::factory(),
            'name' => fake()->words(2, true),
            'discord_webhook_url' => 'https://discord.com/api/webhooks/'.fake()->randomNumber(8, true).'/'.fake()->sha1(),
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
