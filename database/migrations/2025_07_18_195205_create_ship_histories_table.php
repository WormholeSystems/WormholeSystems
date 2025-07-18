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
        Schema::create('ship_histories', function (Blueprint $table) {
            $table->id();
            $table->unsignedBigInteger('ship_id')->index();
            $table->foreignId('ship_type_id')->constrained('types')->onDelete('cascade');
            $table->foreignId('character_id')->constrained('characters')->onDelete('cascade');
            $table->string('name');
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('ship_histories');
    }
};
