<?php

declare(strict_types=1);

namespace App\DTO;

final readonly class OrganisationStats
{
    public function __construct(
        public string $type,
        public int $killCount,
        public array $activeDays,
    ) {}

    public static function create(string $type): self
    {
        return new self(
            type: $type,
            killCount: 0,
            activeDays: [],
        );
    }

    public function getActiveDayCount(): int
    {
        return count($this->activeDays);
    }

    public function addKill(string $date): self
    {
        $activeDays = $this->activeDays;
        $activeDays[$date] = true;

        return new self(
            type: $this->type,
            killCount: $this->killCount + 1,
            activeDays: $activeDays,
        );
    }
}
