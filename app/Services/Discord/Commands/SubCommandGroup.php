<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

/**
 * Fluent builder for a Discord subcommand group: a named container of subcommands.
 */
final class SubCommandGroup
{
    /** @var SubCommand[] */
    private array $subCommands = [];

    private function __construct(
        private readonly string $name,
        private readonly string $description,
    ) {}

    public static function make(string $name, string $description): self
    {
        return new self($name, $description);
    }

    public function subCommands(SubCommand ...$subCommands): self
    {
        $this->subCommands = [...$this->subCommands, ...$subCommands];

        return $this;
    }

    /** @return array<string, mixed> */
    public function toArray(): array
    {
        return [
            'type' => 2,
            'name' => $this->name,
            'description' => $this->description,
            'options' => array_map(fn (SubCommand $subCommand): array => $subCommand->toArray(), $this->subCommands),
        ];
    }
}
