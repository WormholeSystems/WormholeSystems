<?php

namespace Database\Factories;

use App\Models\Corporation;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<Corporation>
 */
class CorporationFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'id' => $this->faker->unique()->randomFloat(0, 1, 10_000_000_000),
            'name' => $this->faker->company(),
        ];
    }
}
