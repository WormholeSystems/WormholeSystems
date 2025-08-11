<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\Map;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<Map>
 */
final class MapFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'name' => $this->faker->name(),
        ];
    }
}
