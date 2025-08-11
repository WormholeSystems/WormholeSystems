<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard\DTO;

final readonly class zkb
{
    public function __construct(
        public int $locationID,
        public string $hash,
        public float $fittedValue,
        public float $droppedValue,
        public float $destroyedValue,
        public float $totalValue,
        public float $points,
        public bool $npc,
        public bool $solo,
        public bool $awox,
        public array $labels,
        public string $href,
    ) {}

    public static function fromArray(array $data): self
    {
        return new self(
            locationID: $data['locationID'] ?? 0,
            hash: $data['hash'] ?? '',
            fittedValue: $data['fittedValue'] ?? 0.0,
            droppedValue: $data['droppedValue'] ?? 0.0,
            destroyedValue: $data['destroyedValue'] ?? 0.0,
            totalValue: $data['totalValue'] ?? 0.0,
            points: $data['points'] ?? 0.0,
            npc: $data['npc'] ?? false,
            solo: $data['solo'] ?? false,
            awox: $data['awox'] ?? false,
            labels: $data['labels'] ?? [],
            href: $data['href'] ?? '',
        );
    }
}
