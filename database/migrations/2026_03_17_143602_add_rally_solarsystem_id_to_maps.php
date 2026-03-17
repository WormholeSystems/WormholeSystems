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
        Schema::table('maps', function (Blueprint $table): void {
            $table->foreignId('rally_solarsystem_id')->nullable()->constrained('solarsystems')->nullOnDelete();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('maps', function (Blueprint $table): void {
            $table->dropConstrainedForeignId('rally_solarsystem_id');
        });
    }
};
