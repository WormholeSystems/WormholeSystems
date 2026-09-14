<?php

declare(strict_types=1);

namespace App\DTO;

final readonly class MaintainerEntry
{
    /**
     * @param  list<MaintainerCharacterStat>  $characters
     */
    public function __construct(
        public int $position,
        public int $points,
        public ?int $user_id,
        public string $display_name,
        public array $characters,
    ) {}
}
