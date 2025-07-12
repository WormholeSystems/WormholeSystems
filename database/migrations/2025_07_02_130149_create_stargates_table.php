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
        Schema::create('stargates', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->foreignId('solarsystem_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('constellation_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('region_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('type_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('destination_id')->nullable()->constrained('stargates')->nullOnDelete()->cascadeOnUpdate();
            $table->bigInteger('position_x')->nullable();
            $table->bigInteger('position_y')->nullable();
            $table->bigInteger('position_z')->nullable();
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
