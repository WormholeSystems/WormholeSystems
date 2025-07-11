<?php

namespace Database\Factories;

use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\Character>
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
            'id' => $this->faker->randomNumber(7),
            'name' => $this->faker->name(),
            'corporation_id' => Corporation::factory(),
            'alliance_id' => Alliance::factory(),
            'user_id' => User::factory(),
        ];
    }
}
