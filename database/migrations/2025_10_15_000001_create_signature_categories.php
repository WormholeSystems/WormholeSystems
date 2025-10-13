<?php

declare(strict_types=1);

use Database\Seeders\SignatureCategorySeeder;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        // Create signature_categories table
        Schema::create('signature_categories', function (Blueprint $table): void {
            $table->id();
            $table->string('name')->unique();
            $table->string('code')->unique();
            $table->timestamps();
        });

        // Seed signature_categories
        Artisan::call('db:seed', ['--class' => SignatureCategorySeeder::class, '--force' => true]);

        // Add signature_category_id to signatures table
        Schema::table('signatures', function (Blueprint $table): void {
            $table->foreignId('signature_category_id')
                ->nullable()
                ->after('wormhole_id')
                ->constrained('signature_categories')
                ->onDelete('set null');
        });

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

        // Remove old category and name columns
        Schema::table('signatures', function (Blueprint $table): void {
            $table->dropColumn(['category', 'name']);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        // Restore old columns
        Schema::table('signatures', function (Blueprint $table): void {
            $table->string('name')->nullable()->after('signature_id');
            $table->string('category')->nullable()->after('signature_id');
        });

        // Restore category data
        $categories = DB::table('signature_categories')->get();
        foreach ($categories as $category) {
            DB::table('signatures')
                ->where('signature_category_id', $category->id)
                ->update(['category' => $category->name]);
        }

        // Remove signature_category_id
        Schema::table('signatures', function (Blueprint $table): void {
            $table->dropForeign(['signature_category_id']);
            $table->dropColumn('signature_category_id');
        });

        // Drop signature_categories table
        Schema::dropIfExists('signature_categories');
    }
};
