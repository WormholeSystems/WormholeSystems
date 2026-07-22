<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

/**
 * Fluent builder for a Discord subcommand: the invocable leaf carrying the options.
 */
final class SubCommand
{
    /** @var Option[] */
    private array $options = [];

    private function __construct(
        private readonly string $name,
        private readonly string $description,
    ) {}

    public static function make(string $name, string $description): self
    {
        return new self($name, $description);
    }

    public function options(Option ...$options): self
    {
        $this->options = [...$this->options, ...$options];

        return $this;
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return array_filter([
            'type' => 1,
            'name' => $this->name,
            'description' => $this->description,
            'options' => $this->options !== []
                ? array_map(fn (Option $option): array => $option->toArray(), $this->options)
                : null,
        ], fn (mixed $value): bool => $value !== null);
    }
}
