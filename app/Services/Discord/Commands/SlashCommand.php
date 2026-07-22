<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

/**
 * Fluent builder for a top-level Discord application command.
 */
final class SlashCommand
{
    /** @var array<int, SubCommandGroup|SubCommand|Option> */
    private array $options = [];

    private function __construct(
        private readonly string $name,
        private readonly string $description,
    ) {}

    public static function make(string $name, string $description): self
    {
        return new self($name, $description);
    }

    public function name(): string
    {
        return $this->name;
    }

    public function options(SubCommandGroup|SubCommand|Option ...$options): self
    {
        $this->options = [...$this->options, ...$options];

        return $this;
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return [
            'type' => 1,
            'name' => $this->name,
            'description' => $this->description,
            'options' => array_map(
                fn (SubCommandGroup|SubCommand|Option $option): array => $option->toArray(),
                $this->options,
            ),
        ];
    }
}
