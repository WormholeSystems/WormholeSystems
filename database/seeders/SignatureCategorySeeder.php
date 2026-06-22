<?php

declare(strict_types=1);

namespace Database\Seeders;

use App\Enums\SignatureCategory as SignatureCategoryEnum;
use App\Models\SignatureCategory;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\File;

final class SignatureCategorySeeder extends Seeder
{
    /**
     * Run the database seeds.
     *
     * Categories are seeded directly from the source-of-truth JSON file
     * (resources/js/data/signatures.json), which the frontend also imports.
     * IDs are preserved exactly so existing signature foreign keys keep resolving.
     */
    public function run(): void
    {
        $path = resource_path('js/data/signatures.json');
        $data = json_decode(File::get($path), true, flags: JSON_THROW_ON_ERROR);

        foreach ($data['categories'] as $category) {
            $model = SignatureCategory::firstOrNew(['id' => $category['id']]);
            $model->id = $category['id'];
            $model->name = $category['name'];
            $model->code = SignatureCategoryEnum::from($category['code']);
            $model->save();
        }
    }
}
