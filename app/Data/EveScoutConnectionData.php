<?php

declare(strict_types=1);

namespace App\Data;

use Spatie\LaravelData\Data;

final class EveScoutConnectionData extends Data
{
    public function __construct(
        public int $in_system_id,
        public int $out_system_id,
        public string $in_signature,
        public string $out_signature,
        public string $wormhole_type,
        public string $life,
        public string $mass,
        public ?float $remaining_hours,
        public ?string $created_at,
    ) {}
}
