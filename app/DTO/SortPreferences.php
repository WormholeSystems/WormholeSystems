<?php

declare(strict_types=1);

namespace App\DTO;

use Illuminate\Contracts\Support\Arrayable;

final class SortPreferences implements Arrayable
{
    public function __construct(
        public SortPreference $signatures
    ) {}

    public static function fromArray(array $data): self
    {
        return new self(
            signatures: SortPreference::from($data['signatures'] ?? []),
        );
    }

    public function toArray(): array
    {
        return [
            'signatures' => $this->signatures->toArray(),
        ];
    }
}
