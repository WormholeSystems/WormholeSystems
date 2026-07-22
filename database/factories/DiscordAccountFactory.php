<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\DiscordAccount;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<DiscordAccount>
 */
final class DiscordAccountFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'discord_user_id' => (string) fake()->unique()->numberBetween(100000000000000000, 999999999999999999),
            'username' => fake()->userName(),
            'display_name' => fake()->name(),
            'avatar' => null,
        ];
    }
}
