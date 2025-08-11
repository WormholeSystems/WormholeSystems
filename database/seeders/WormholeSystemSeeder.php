<?php

declare(strict_types=1);

namespace Database\Seeders;

use App\Models\Wormhole;
use App\Models\WormholeEffect;
use App\Models\WormholeStatic;
use App\Models\WormholeSystem;
use Carbon\Carbon;
use Illuminate\Database\Seeder;
use RuntimeException;

final class WormholeSystemSeeder extends Seeder
{
    public const string csv = __DIR__.'/data/wormhole_systems.csv';

    /**
     * Run the database seeds.
     */
    public function run(): void
    {
        if (! file_exists(self::csv)) {
            throw new RuntimeException('Wormhole system CSV file not found: '.self::csv);
        }

        $data = array_map('str_getcsv', file(self::csv));

        if ($data === []) {
            throw new RuntimeException('No data found in wormhole system CSV file: '.self::csv);
        }

        $header = array_shift($data);

        foreach ($data as $row) {
            if (count($row) !== count($header)) {
                throw new RuntimeException('Row length does not match header length in wormhole system CSV file: '.self::csv);
            }

            $rowData = array_combine($header, $row);

            WormholeSystem::query()->updateOrCreate(
                [
                    'id' => $rowData['id'],
                ],
                [
                    'effect_id' => WormholeEffect::query()
                        ->where('name', $rowData['effect'])
                        ->value('id'),
                    'class' => $rowData['class'],
                    'created_at' => Carbon::now(),
                    'updated_at' => Carbon::now(),
                ]
            );

            foreach (explode(',', (string) $rowData['statics']) as $staticName) {
                $staticName = mb_trim($staticName);
                if ($staticName === '') {
                    continue;
                }
                if ($staticName === '0') {
                    continue;
                }

                $wormhole = Wormhole::query()->where('name', $staticName)->first();
                if (! $wormhole) {
                    throw new RuntimeException('Wormhole not found for static: '.$staticName);
                }

                WormholeStatic::query()->updateOrCreate(
                    [
                        'wormhole_id' => $wormhole->id,
                        'wormhole_system_id' => $rowData['id'],
                    ]
                );
            }
        }
    }
}
