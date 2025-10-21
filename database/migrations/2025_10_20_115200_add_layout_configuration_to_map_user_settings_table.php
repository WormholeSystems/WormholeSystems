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
        Schema::table('map_user_settings', function (Blueprint $table): void {
            $table->json('layout_config_sm')->nullable();
            $table->json('layout_config_md')->nullable();
            $table->json('layout_config_lg')->nullable();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_user_settings', function (Blueprint $table): void {
            $table->dropColumn([
                'layout_config_sm',
                'layout_config_md',
                'layout_config_lg',
            ]);
        });
    }
};
