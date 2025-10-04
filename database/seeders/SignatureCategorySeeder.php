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
        foreach (\App\Enums\SignatureCategory::cases() as $category) {
            SignatureCategory::firstOrCreate([
                'code' => $category->value,
                'name' => $category->name(),
            ]);
        }
    }
}
