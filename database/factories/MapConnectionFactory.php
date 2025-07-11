<?php

namespace Database\Factories;

use App\Enums\MassStatus;
use App\Enums\ShipSize;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapConnection>
 */
class MapConnectionFactory extends Factory
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
            'from_map_solarsystem_id' => MapSolarsystem::factory(),
            'to_map_solarsystem_id' => MapSolarsystem::factory(),
            'mass_status' => $this->faker->randomElement(MassStatus::cases()),
            'is_eol' => $this->faker->boolean(),
            'ship_size' => $this->faker->randomElement(ShipSize::cases()),
        ];
    }
}
