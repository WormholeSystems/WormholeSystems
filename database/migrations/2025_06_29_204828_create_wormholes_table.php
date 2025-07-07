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
        Schema::create('wormholes', function (Blueprint $table) {
            $table->id();
            $table->string('name')->index();
            $table->unsignedBigInteger('total_mass');
            $table->unsignedBigInteger('maximum_jump_mass');
            $table->string('ship_size')->nullable()->index();
            $table->unsignedBigInteger('maximum_lifetime')->nullable();
            $table->string('leads_to')->nullable()->index();
            $table->foreignId('type_id')->nullable()->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('wormholes');
    }
};
