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
        Schema::create('wormhole_systems', function (Blueprint $table) {
            $table->foreignId('id')->primary()->constrained()->references('id')->on('solarsystems')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('effect_id')->nullable()->constrained('wormhole_effects')->cascadeOnDelete()->cascadeOnUpdate();
            $table->unsignedBigInteger('class')->nullable()->index();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('wormhole_systems');
    }
};
