<?php

declare(strict_types=1);

namespace Database\Seeders;

use App\Models\SignatureCategory;
use Illuminate\Database\Seeder;

final class SignatureCategorySeeder extends Seeder
{
    /**
     * Run the database seeds.
     */
    public function run(): void
    {
        $categories = [
            ['name' => 'Wormhole', 'code' => 'wormhole'],
            ['name' => 'Data Site', 'code' => 'data'],
            ['name' => 'Relic Site', 'code' => 'relic'],
            ['name' => 'Combat Site', 'code' => 'combat'],
            ['name' => 'Gas Site', 'code' => 'gas'],
            ['name' => 'Ore Site', 'code' => 'ore'],
        ];

        foreach ($categories as $category) {
            SignatureCategory::updateOrCreate(
                ['code' => $category['code']],
                $category
            );
        }
    }
}
