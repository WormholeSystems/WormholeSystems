<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\MapSolarsystemDetails;
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
            'position_x' => (int) $this->faker->randomFloat(0, 0, config('map.max_size.x')),
            'position_y' => (int) $this->faker->randomFloat(0, 0, config('map.max_size.y')),
            'pinned' => $this->faker->boolean(),
            'solarsystem_id' => fn () => Solarsystem::query()->inRandomOrder()->first()->id,
        ];
    }

    /**
     * Ensure every placement is backed by a persistent details row that shares its
     * (map_id, solarsystem_id), unless one was provided explicitly.
     */
    public function configure(): static
    {
        return $this->afterMaking(function (MapSolarsystem $mapSolarsystem): void {
            if ($mapSolarsystem->map_solarsystem_details_id !== null) {
                return;
            }

            $details = MapSolarsystemDetails::factory()->create([
                'map_id' => $mapSolarsystem->map_id,
                'solarsystem_id' => $mapSolarsystem->solarsystem_id,
            ]);

            $mapSolarsystem->map_solarsystem_details_id = $details->id;
        });
    }
}
