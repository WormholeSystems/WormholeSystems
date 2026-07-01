<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\Map;
use App\Models\MapWebhookRole;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapWebhookRole>
 */
final class MapWebhookRoleFactory extends Factory
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
            'discord_role_id' => (string) fake()->randomNumber(9, true),
        ];
    }
}
