<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\Character;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapConnectionJump;
use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapConnectionJump>
 */
final class MapConnectionJumpFactory extends Factory
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
            'map_connection_id' => MapConnection::factory(),
            'character_id' => Character::factory(),
            'from_solarsystem_id' => fn () => Solarsystem::query()->inRandomOrder()->first()->id,
            'to_solarsystem_id' => fn () => Solarsystem::query()->inRandomOrder()->first()->id,
            'ship_type_id' => null,
            'ship_name' => $this->faker->word(),
            'mass' => $this->faker->numberBetween(1_000_000, 300_000_000),
        ];
    }

    public function pending(): static
    {
        return $this->state(fn (): array => ['map_connection_id' => null]);
    }
}
