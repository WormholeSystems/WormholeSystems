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
        Schema::create('raidable_skyhooks', function (Blueprint $table): void {
            $table->unsignedBigInteger('planet_id')->primary();
            $table->foreignId('solarsystem_id')->constrained()->cascadeOnDelete();
            $table->dateTime('theft_vulnerability_start');
            $table->dateTime('theft_vulnerability_end')->index();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('raidable_skyhooks');
    }
};
