<?php

namespace Database\Factories;

use App\Models\Alliance;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<Character>
 */
class CharacterFactory extends Factory
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
            'name' => $this->faker->name(),
            'corporation_id' => Corporation::factory(),
            'alliance_id' => Alliance::factory(),
            'user_id' => User::factory(),
        ];
    }
}
