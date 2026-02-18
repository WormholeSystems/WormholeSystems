<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        // Rename permission values in map_access table
        DB::table('map_access')->where('permission', 'guest')->update(['permission' => 'viewer']);
        DB::table('map_access')->where('permission', 'read')->update(['permission' => 'viewer']);
        DB::table('map_access')->where('permission', 'write')->update(['permission' => 'manager']);

        // Add public map and share token columns to maps table
        Schema::table('maps', function (Blueprint $table): void {
            $table->boolean('is_public')->default(false)->after('name');
            $table->string('share_token')->nullable()->unique()->after('is_public');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        // Revert permission values
        DB::table('map_access')->where('permission', 'viewer')->update(['permission' => 'guest']);
        DB::table('map_access')->where('permission', 'member')->update(['permission' => 'read']);
        DB::table('map_access')->where('permission', 'manager')->update(['permission' => 'write']);

        Schema::table('maps', function (Blueprint $table): void {
            $table->dropColumn(['is_public', 'share_token']);
        });
    }
};
