<?php

declare(strict_types=1);

use App\Enums\SignatureCategory as SignatureCategoryEnum;
use App\Models\SignatureCategory;
use Illuminate\Database\Migrations\Migration;

return new class extends Migration
{
    /**
     * Run the migrations.
     *
     * The id is pinned to 8 so it matches resources/js/data/signatures.json,
     * which the frontend uses to resolve category ids when parsing pastes.
     */
    public function up(): void
    {
        $category = SignatureCategoryEnum::FactionWarfare;

        SignatureCategory::firstOrCreate([
            'code' => $category->value,
        ], [
            'id' => 8,
            'name' => $category->name(),
        ]);
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        SignatureCategory::where('code', SignatureCategoryEnum::FactionWarfare->value)->delete();
    }
};
