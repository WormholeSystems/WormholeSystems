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
        Schema::create('killmails', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('hash');
            $table->foreignId('solarsystem_id')->constrained()->cascadeOnDelete();
            $table->dateTime('time')->index();
            $table->json('data');
            $table->json('zkb');
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('killmails');
    }
};
