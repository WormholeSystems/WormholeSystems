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
        Schema::create('graphics', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('sof_faction_name')->nullable()->index();
            $table->string('file')->nullable();
            $table->string('sof_hull_name')->nullable()->index();
            $table->string('sof_race_name')->nullable()->index();
            $table->text('description')->nullable();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('graphics');
    }
};
