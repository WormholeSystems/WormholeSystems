<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::table('map_user_settings', function (Blueprint $table) {
            $table->boolean('copy_bookmark_enabled')->default(false)->after('suggest_alias_enabled');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_user_settings', function (Blueprint $table) {
            $table->dropColumn('copy_bookmark_enabled');
        });
    }
};
