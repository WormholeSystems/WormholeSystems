<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Enums\SignatureCategory;
use App\Enums\SolarsystemClass;
use App\Enums\WormholeSignature;
use App\Models\SignatureType;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<SignatureType>
 */
final class SignatureTypeFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        $category = $this->faker->randomElement(SignatureCategory::cases());

        return [
            'name' => $this->faker->words(3, true),
            'signature' => $category === SignatureCategory::Wormhole ? $this->faker->randomElement(WormholeSignature::cases()) : null,
            'category' => $category,
            'target_class' => $category === SignatureCategory::Wormhole ? $this->faker->randomElement(SolarsystemClass::cases()) : null,
            'extra' => $this->faker->optional(0.3)->word(),
            'spawn_areas' => $this->faker->randomElements(SolarsystemClass::cases(), $this->faker->numberBetween(1, 4)),
        ];
    }

    /**
     * Create a wormhole signature type.
     */
    public function wormhole(): static
    {
        return $this->state(fn (array $attributes): array => [
            'category' => SignatureCategory::Wormhole,
            'signature' => $this->faker->randomElement(WormholeSignature::cases()),
            'target_class' => $this->faker->randomElement(SolarsystemClass::cases()),
        ]);
    }

    /**
     * Create a combat site signature type.
     */
    public function combatSite(): static
    {
        return $this->state(fn (array $attributes): array => [
            'category' => SignatureCategory::CombatSite,
            'signature' => null,
            'target_class' => null,
        ]);
    }

    /**
     * Create a data site signature type.
     */
    public function dataSite(): static
    {
        return $this->state(fn (array $attributes): array => [
            'category' => SignatureCategory::DataSite,
            'signature' => null,
            'target_class' => null,
        ]);
    }

    /**
     * Create a relic site signature type.
     */
    public function relicSite(): static
    {
        return $this->state(fn (array $attributes): array => [
            'category' => SignatureCategory::RelicSite,
            'signature' => null,
            'target_class' => null,
        ]);
    }

    /**
     * Create a gas site signature type.
     */
    public function gasSite(): static
    {
        return $this->state(fn (array $attributes): array => [
            'category' => SignatureCategory::GasSite,
            'signature' => null,
            'target_class' => null,
        ]);
    }

    /**
     * Create an ore site signature type.
     */
    public function oreSite(): static
    {
        return $this->state(fn (array $attributes): array => [
            'category' => SignatureCategory::OreSite,
            'signature' => null,
            'target_class' => null,
        ]);
    }
}
