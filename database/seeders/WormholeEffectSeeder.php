<?php

declare(strict_types=1);

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use RuntimeException;

final class WormholeEffectSeeder extends Seeder
{
    public const string json = __DIR__.'/data/wormhole_effects.json';

    /**
     * Run the database seeds.
     */
    public function run(): void
    {
        if (! file_exists(self::json)) {
            throw new RuntimeException('Wormhole effects JSON file not found: '.self::json);
        }

        $data = json_decode(file_get_contents(self::json), true);

        if (json_last_error() !== JSON_ERROR_NONE) {
            throw new RuntimeException('Invalid JSON in wormhole effects file: '.json_last_error_msg());
        }

        foreach ($data as $effect => $details) {
            \App\Models\WormholeEffect::updateOrCreate(
                ['name' => $effect],
                [
                    'effects' => $details,
                ]
            );
        }
    }
}
