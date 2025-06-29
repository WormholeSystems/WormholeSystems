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
        Schema::create('stations', function (Blueprint $table) {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->unique()->index();
            $table->foreignId('solarsystem_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('constellation_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('region_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('parent_id')->nullable()->constrained('celestials')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('type_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
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
