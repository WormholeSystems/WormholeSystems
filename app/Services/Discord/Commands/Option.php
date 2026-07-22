<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

/**
 * Fluent builder for a single Discord application command option.
 */
final class Option
{
    /** @var array<int, array{name: string, value: string}> */
    private array $choices = [];

    private bool $required = false;

    private bool $autocomplete = false;

    private ?int $minValue = null;

    private ?int $maxValue = null;

    private string $description = '';

    private function __construct(
        private readonly int $type,
        private readonly string $name,
    ) {}

    public static function string(string $name): self
    {
        return new self(3, $name);
    }

    public static function integer(string $name): self
    {
        return new self(4, $name);
    }

    public static function boolean(string $name): self
    {
        return new self(5, $name);
    }

    public static function role(string $name): self
    {
        return new self(8, $name);
    }

    public function description(string $description): self
    {
        $this->description = $description;

        return $this;
    }

    public function required(): self
    {
        $this->required = true;

        return $this;
    }

    public function autocomplete(): self
    {
        $this->autocomplete = true;

        return $this;
    }

    public function between(int $min, int $max): self
    {
        $this->minValue = $min;
        $this->maxValue = $max;

        return $this;
    }

    public function choice(string $name, string $value): self
    {
        $this->choices[] = ['name' => $name, 'value' => $value];

        return $this;
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'type' => $this->type,
            'name' => $this->name,
            'description' => $this->description,
            'required' => $this->required,
            'autocomplete' => $this->autocomplete ?: null,
            'min_value' => $this->minValue,
            'max_value' => $this->maxValue,
            'choices' => $this->choices !== [] ? $this->choices : null,
        ], fn (mixed $value): bool => $value !== null);
    }
}
