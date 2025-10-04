<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Support\Facades\DB;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        // Map category string values to their IDs
        $categoryMap = [
            'Wormhole' => DB::table('signature_categories')->where('code', 'wormhole')->value('id'),
            'Data Site' => DB::table('signature_categories')->where('code', 'data')->value('id'),
            'Relic Site' => DB::table('signature_categories')->where('code', 'relic')->value('id'),
            'Combat Site' => DB::table('signature_categories')->where('code', 'combat')->value('id'),
            'Gas Site' => DB::table('signature_categories')->where('code', 'gas')->value('id'),
            'Ore Site' => DB::table('signature_categories')->where('code', 'ore')->value('id'),
        ];

        // Update signatures with their corresponding category IDs
        foreach ($categoryMap as $categoryName => $categoryId) {
            if ($categoryId) {
                DB::table('signatures')
                    ->where('category', $categoryName)
                    ->update(['signature_category_id' => $categoryId]);
            }
        }
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        // Get the reverse mapping (category ID to name)
        $categories = DB::table('signature_categories')->get();
        
        foreach ($categories as $category) {
            DB::table('signatures')
                ->where('signature_category_id', $category->id)
                ->update(['category' => $category->name]);
        }
    }
};
