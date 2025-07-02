<?php

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
        Schema::create('map_connections', function (Blueprint $table) {
            $table->id();
            $table->foreignId('map_id')
                ->constrained('maps')
                ->cascadeOnDelete()
                ->cascadeOnUpdate();
            $table->foreignId('from_map_solarsystem_id')
                ->constrained('map_solarsystems')
                ->cascadeOnDelete()
                ->cascadeOnUpdate();
            $table->foreignId('to_map_solarsystem_id')
                ->constrained('map_solarsystems')
                ->cascadeOnDelete()
                ->cascadeOnUpdate();
            $table->foreignId('wormhole_id')->nullable()->constrained('wormholes')->cascadeOnDelete()->cascadeOnUpdate();
            $table->string('status')->nullable()->index();
            $table->dateTime('connected_at')->nullable();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('map_connections');
    }
};
