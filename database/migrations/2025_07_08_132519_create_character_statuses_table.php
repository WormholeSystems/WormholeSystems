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
        Schema::create('character_statuses', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('character_id')->constrained()->onDelete('cascade');
            $table->foreignId('solarsystem_id')->nullable()->constrained()->nullOnDelete();
            $table->foreignId('station_id')->nullable()->constrained()->nullOnDelete();
            $table->unsignedBigInteger('structure_id')->nullable();
            $table->foreignId('ship_type_id')->nullable()->constrained('types')->nullOnDelete();
            $table->unsignedBigInteger('ship_item_id')->nullable();
            $table->string('ship_name')->nullable();
            $table->boolean('is_online')->default(false);
            $table->dateTime('last_online_at')->nullable();
            $table->dateTime('online_last_checked_at')->nullable();
            $table->dateTime('location_last_checked_at')->nullable();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('character_statuses');
    }
};
