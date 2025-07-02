<?php

namespace Database\Seeders;

use App\Models\Wormhole;
use Carbon\Carbon;
use Illuminate\Database\Console\Seeds\WithoutModelEvents;
use Illuminate\Database\Seeder;

class WormholeSeeder extends Seeder
{
    const string json = __DIR__ . '/data/wormholes.json';
    /**
     * Run the database seeds.
     */
    public function run(): void
    {
        if (!file_exists(self::json)) {
            throw new \RuntimeException('Wormhole JSON file not found: ' . self::json);
        }

        $data = json_decode(file_get_contents(self::json), true);

        if (json_last_error() !== JSON_ERROR_NONE) {
            throw new \RuntimeException('Invalid JSON in wormhole file: ' . json_last_error_msg());
        }

        foreach ($data as $name => $details) {
            Wormhole::query()->updateOrCreate([
                'name' => $name
            ], [
                'name' =>  $name,
                'leads_to' =>  $details['dest'],
                'total_mass' =>   $details['total_mass'] ?? 0,
                'type_id' =>  $details['typeID'],
                'maximum_jump_mass' =>    $details['max_mass_per_jump'] ?? 0,
                'maximum_lifetime' => $details['lifetime'] * 60 * 60, // Convert from hours to seconds
            ]);
        }
    }
}
