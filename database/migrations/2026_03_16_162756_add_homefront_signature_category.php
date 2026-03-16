<?php

declare(strict_types=1);

use App\Enums\SignatureCategory as SignatureCategoryEnum;
use App\Models\SignatureCategory;
use Illuminate\Database\Migrations\Migration;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        $category = SignatureCategoryEnum::Homefront;

        SignatureCategory::firstOrCreate([
            'code' => $category->value,
        ], [
            'name' => $category->name(),
        ]);
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        SignatureCategory::where('code', 'homefront')->delete();
    }
};
