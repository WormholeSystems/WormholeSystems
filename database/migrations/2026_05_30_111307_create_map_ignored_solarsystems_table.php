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
        Schema::create('map_ignored_solarsystems', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')->constrained()->onDelete('cascade');
            $table->foreignId('solarsystem_id')->constrained()->onDelete('cascade');
            $table->unique(['map_id', 'solarsystem_id']);
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('map_ignored_solarsystems');
    }
};
