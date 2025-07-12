<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('solarsystem_connections', function (Blueprint $table) {
            $table->foreignId('from_stargate_id')->primary()->constrained('stargates')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('from_solarsystem_id')->constrained('solarsystems')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('from_constellation_id')->constrained('constellations')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('from_region_id')->constrained('regions')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('to_stargate_id')->constrained('stargates')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('to_solarsystem_id')->constrained('solarsystems')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('to_constellation_id')->constrained('constellations')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('to_region_id')->constrained('regions')->cascadeOnDelete()->cascadeOnUpdate();
            $table->boolean('is_regional')->default(false);
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('stations');
    }
};
