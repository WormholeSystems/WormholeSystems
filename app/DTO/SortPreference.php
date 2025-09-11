<?php

declare(strict_types=1);

namespace App\DTO;

use App\Enums\SortDirection;
use Illuminate\Contracts\Support\Arrayable;

final class SortPreference implements Arrayable
{
    public function __construct(
        public string $column,
        public SortDirection $direction = SortDirection::ASC,
    ) {}

    public static function from(array $data): self
    {
        return new self(
            column: $data['column'] ?? 'id',
            direction: SortDirection::tryFrom($data['direction'] ?? 'asc') ?? SortDirection::ASC,
        );
    }

    public function toArray(): array
    {
        return [
            'column' => $this->column,
            'direction' => $this->direction->value,
        ];
    }
}
