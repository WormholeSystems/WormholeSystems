<?php

declare(strict_types=1);

use App\Enums\SignatureCategory as SignatureCategoryEnum;
use App\Enums\SolarsystemClass;
use App\Models\SignatureCategory;
use App\Models\SignatureType;
use Illuminate\Database\Migrations\Migration;

return new class extends Migration
{
    /**
     * Run the migrations.
     *
     * The id is pinned to 272 so it matches resources/js/data/signatures.json,
     * which the frontend uses to resolve signature type ids.
     */
    public function up(): void
    {
        if (SignatureType::query()->whereKey(272)->exists()) {
            return;
        }

        $type = new SignatureType;
        $type->id = 272;
        $type->name = 'Rampant Drone Fabricator';
        $type->signature_category_id = SignatureCategory::query()
            ->where('code', SignatureCategoryEnum::Combat->value)
            ->value('id');
        $type->spawn_areas = [
            SolarsystemClass::C1,
            SolarsystemClass::C2,
            SolarsystemClass::C3,
            SolarsystemClass::C4,
            SolarsystemClass::C5,
            SolarsystemClass::C6,
        ];
        $type->save();
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        SignatureType::query()->whereKey(272)->delete();
    }
};
