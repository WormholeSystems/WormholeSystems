<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\Map;
use App\Models\MapIgnoredSolarsystem;
use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapIgnoredSolarsystem>
 */
final class MapIgnoredSolarsystemFactory extends Factory
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
            'solarsystem_id' => fn () => Solarsystem::query()->inRandomOrder()->first()->id,
        ];
    }
}
