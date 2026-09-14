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
        Schema::create('map_maintainer_reports', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')->constrained('maps')->cascadeOnDelete();
            $table->char('period', 7);
            $table->json('payload');
            $table->timestamp('generated_at');
            // Idempotency marker for S5's monthly Discord post -- per-period, not per-alert,
            // so a manual re-run of an already-posted period is correctly suppressed.
            $table->timestamp('discord_posted_at')->nullable();
            $table->timestamps();

            $table->unique(['map_id', 'period']);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('map_maintainer_reports');
    }
};
