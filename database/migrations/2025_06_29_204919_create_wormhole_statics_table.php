<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('wormhole_statics', function (Blueprint $table) {
            $table->id();
            $table->foreignId('wormhole_system_id')
                ->constrained('wormhole_systems')
                ->cascadeOnDelete()
                ->cascadeOnUpdate();
            $table->foreignId('wormhole_id')
                ->constrained('wormhole_effects')
                ->cascadeOnDelete()
                ->cascadeOnUpdate();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('wormhole_statics');
    }
};
