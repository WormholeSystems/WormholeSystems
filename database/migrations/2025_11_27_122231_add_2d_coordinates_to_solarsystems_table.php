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
        Schema::table('solarsystems', function (Blueprint $table): void {
            $table->double('pos_2d_x')->nullable()->after('pos_z');
            $table->double('pos_2d_y')->nullable()->after('pos_2d_x');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('solarsystems', function (Blueprint $table): void {
            $table->dropColumn(['pos_2d_x', 'pos_2d_y']);
        });
    }
};
