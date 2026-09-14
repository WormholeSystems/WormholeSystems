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
        Schema::create('signature_activities', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')->constrained('maps')->cascadeOnDelete();
            // unsignedBigInteger, not foreignId(): characters.id is an EVE character id, non-incrementing.
            $table->unsignedBigInteger('character_id');
            $table->foreign('character_id')->references('id')->on('characters')->restrictOnDelete();
            // Deliberately not a foreign key: the row must outlive the signature it refers to.
            $table->unsignedBigInteger('signature_id');
            $table->unsignedBigInteger('solarsystem_id');
            $table->string('dedupe_key', 64);
            $table->string('action', 16);
            $table->date('activity_date');
            // No ->useCurrent(): always written explicitly, from the same app-clock instant as
            // activity_date, so the churn window and the day bucket never straddle two clocks.
            $table->timestamp('created_at');

            $table->unique(
                ['map_id', 'character_id', 'solarsystem_id', 'dedupe_key', 'action', 'activity_date'],
                'signature_activities_daily_unique',
            );
            $table->index(['map_id', 'activity_date']);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('signature_activities');
    }
};
