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
        Schema::create('map_connection_jumps', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')->constrained('maps')->cascadeOnDelete();
            $table->foreignId('map_connection_id')->nullable()->constrained('map_connections')->cascadeOnDelete();
            $table->foreignId('character_id')->constrained('characters')->cascadeOnDelete();
            $table->foreignId('from_solarsystem_id')->constrained('solarsystems');
            $table->foreignId('to_solarsystem_id')->constrained('solarsystems');
            $table->foreignId('ship_type_id')->nullable()->constrained('types')->nullOnDelete();
            $table->string('ship_name')->nullable();
            $table->unsignedBigInteger('mass')->default(0);
            $table->timestamps();

            $table->index(['map_id', 'from_solarsystem_id', 'to_solarsystem_id', 'created_at'], 'mcj_pending_claim_index');
            $table->index(['map_connection_id', 'created_at']);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('map_connection_jumps');
    }
};
