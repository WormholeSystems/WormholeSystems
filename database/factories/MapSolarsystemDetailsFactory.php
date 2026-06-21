<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Enums\MapSolarsystemStatus;
use App\Models\Map;
use App\Models\MapSolarsystemDetails;
use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapSolarsystemDetails>
 */
final class MapSolarsystemDetailsFactory extends Factory
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
            'occupier_alias' => $this->faker->word(),
            'status' => $this->faker->randomElement(MapSolarsystemStatus::cases()),
            'notes' => null,
            'solarsystem_id' => fn () => Solarsystem::query()->inRandomOrder()->first()->id,
        ];
    }
}
