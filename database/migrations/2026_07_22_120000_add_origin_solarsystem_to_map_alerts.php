<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('map_alerts', function (Blueprint $table): void {
            $table->foreignId('origin_solarsystem_id')->nullable()->after('target_solarsystem_id')->constrained('solarsystems')->nullOnDelete();
        });
    }

    public function down(): void
    {
        Schema::table('map_alerts', function (Blueprint $table): void {
            $table->dropConstrainedForeignId('origin_solarsystem_id');
        });
    }
};
