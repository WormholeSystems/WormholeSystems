<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<User>
 */
final class UserFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'name' => fake()->name(),
        ];
    }

    public function ownsMap(Map $map): self
    {
        return $this->has(Character::factory()->has(MapAccess::factory(['is_owner' => true])->for($map)));
    }
}
