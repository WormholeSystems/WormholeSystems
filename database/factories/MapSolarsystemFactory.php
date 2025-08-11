<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Enums\MapSolarsystemStatus;
use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapSolarsystem>
 */
final class MapSolarsystemFactory extends Factory
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
            'alias' => $this->faker->unique()->word(),
            'occupier_alias' => $this->faker->word(),
            'position_x' => (int) $this->faker->randomFloat(0, 0, config('map.max_size.x')),
            'position_y' => (int) $this->faker->randomFloat(0, 0, config('map.max_size.y')),
            'status' => $this->faker->randomElement(MapSolarsystemStatus::cases()),
            'pinned' => $this->faker->boolean(),
            'solarsystem_id' => fn () => Solarsystem::query()->inRandomOrder()->first()->id,
        ];
    }
}
