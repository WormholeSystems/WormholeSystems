<?php

declare(strict_types=1);

namespace Database\Seeders;

use App\Enums\SolarsystemClass;
use App\Enums\WormholeSignature;
use App\Models\SignatureType;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\File;

final class SignatureTypeSeeder extends Seeder
{
    /**
     * Run the database seeds.
     *
     * Signature types are seeded directly from the source-of-truth JSON file
     * (resources/js/data/signatures.json), which the frontend also imports.
     * IDs are preserved exactly so existing signature foreign keys keep resolving,
     * and every enum-typed field is validated on read so a typo fails loudly.
     */
    public function run(): void
    {
        $path = resource_path('js/data/signatures.json');
        $data = json_decode(File::get($path), true, flags: JSON_THROW_ON_ERROR);

        foreach ($data['types'] as $type) {
            $model = SignatureType::firstOrNew(['id' => $type['id']]);
            $model->id = $type['id'];
            $model->name = $type['name'];
            $model->signature_category_id = $type['signature_category_id'];
            $model->signature = $type['signature'] !== null
                ? WormholeSignature::from($type['signature'])
                : null;
            $model->target_class = $type['target_class'] !== null
                ? SolarsystemClass::from($type['target_class'])
                : null;
            $model->extra = $type['extra'];
            $model->spawn_areas = $type['spawn_areas'] !== null
                ? array_map(SolarsystemClass::from(...), $type['spawn_areas'])
                : null;
            $model->save();
        }
    }
}
