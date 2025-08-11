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
        Schema::create('signatures', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_solarsystem_id')->constrained('map_solarsystems')->onDelete('cascade');
            $table->string('signature_id')->nullable();
            $table->string('category')->nullable();
            $table->string('type')->nullable();
            $table->foreignId('map_connection_id')->nullable()->constrained('map_connections')->onDelete('cascade');
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('signatures');
    }
};
