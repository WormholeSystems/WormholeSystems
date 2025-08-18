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
        Schema::table('signatures', function (Blueprint $table): void {
            $table->foreignId('wormhole_id')->nullable()->after('map_connection_id')->constrained('wormholes')->onDelete('cascade');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('signatures', function (Blueprint $table): void {
            $table->dropForeign(['wormhole_id']);
            $table->dropColumn('wormhole_id');
        });
    }
};
