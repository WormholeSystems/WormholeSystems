<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\MapRouteSolarsystem>
 */
final class MapRouteSolarsystemFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'is_pinned' => false,
        ];
    }

    /**
     * A personal row, owned by the given user (or a new one).
     */
    public function personal(?User $user = null): self
    {
        return $this->state(fn (): array => [
            'user_id' => $user instanceof User ? $user->id : User::factory(),
        ]);
    }

    public function pinned(): self
    {
        return $this->state(fn (): array => [
            'is_pinned' => true,
        ]);
    }
}
