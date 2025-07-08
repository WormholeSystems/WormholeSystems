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
        Schema::create('solarsystems', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->unique()->index();
            $table->foreignId('constellation_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('region_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->double('security');
            $table->double('pos_x');
            $table->double('pos_y');
            $table->double('pos_z');
            $table->string('type')->index();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('solarsystems');
    }
};
