<?php

declare(strict_types=1);

namespace App\DTO;

use App\Models\Alliance;
use App\Models\Corporation;

final readonly class EntityDetails
{
    public function __construct(
        public string $name,
        public int $id,
        public string $type,
        public int $kills,
    ) {}

    public static function fromId(int $id, int $kills): self
    {
        $alliance = Alliance::query()->find($id);
        if ($alliance) {
            return new self(
                name: $alliance->name,
                id: $alliance->id,
                type: 'alliance',
                kills: $kills,
            );
        }

        $corporation = Corporation::query()->find($id);
        if ($corporation) {
            return new self(
                name: $corporation->name,
                id: $corporation->id,
                type: 'corporation',
                kills: $kills,
            );
        }

        return new self(
            name: 'Unknown entity',
            id: $id,
            type: 'unknown',
            kills: $kills,
        );
    }

    public function toMarkdown(): string
    {
        if ($this->type === 'unknown') {
            return sprintf('- %s with ID %d and %d kills', $this->name, $this->id, $this->kills);
        }

        return sprintf(
            '- [%s](https://zkillboard.com/%s/%d/) - %d kills',
            $this->name,
            $this->type,
            $this->id,
            $this->kills
        );
    }
}
