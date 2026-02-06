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
        Schema::create('station_operations', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->index();
            $table->timestamps();
        });

        Schema::create('services', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->index();
            $table->timestamps();
        });

        Schema::create('operation_services', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('station_operation_id')->constrained('station_operations')->cascadeOnDelete();
            $table->foreignId('service_id')->constrained('services')->cascadeOnDelete();
            $table->timestamps();

            $table->unique(['station_operation_id', 'service_id']);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('operation_services');
        Schema::dropIfExists('services');
        Schema::dropIfExists('station_operations');
    }
};
