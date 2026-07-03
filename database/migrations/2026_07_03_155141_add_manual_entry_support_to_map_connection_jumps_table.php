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
        Schema::table('map_connection_jumps', function (Blueprint $table): void {
            $table->foreignId('character_id')->nullable()->change();
            $table->boolean('is_manual')->default(false)->after('mass');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_connection_jumps', function (Blueprint $table): void {
            $table->foreignId('character_id')->nullable(false)->change();
            $table->dropColumn('is_manual');
        });
    }
};
