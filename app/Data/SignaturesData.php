<?php

declare(strict_types=1);

namespace App\Data;

use App\Models\MapSolarsystem;
use Spatie\LaravelData\Attributes\Computed;
use Spatie\LaravelData\Attributes\Validation\Exists;
use Spatie\LaravelData\Data;

/**
 * @property RawSignatureData[] $signatures
 */
final class SignaturesData extends Data
{
    #[Computed]
    public MapSolarsystem $mapSolarsystem;

    public function __construct(
        #[Exists(table: 'map_solarsystems', column: 'id')]
        public int $map_solarsystem_id,
        public array $signatures,
    ) {
        $this->mapSolarsystem = MapSolarsystem::query()->findOrFail($this->map_solarsystem_id);
    }
}
