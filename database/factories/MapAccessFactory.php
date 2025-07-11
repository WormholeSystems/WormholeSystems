<?php

namespace Database\Factories;

use App\Models\Map;
use App\Models\MapAccess;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapAccess>
 */
class MapAccessFactory extends Factory
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
        ];
    }
}
