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
        Schema::create('type_attributes', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('type_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('attribute_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->double('value')->nullable();
            $table->unique(['type_id', 'attribute_id']);
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('type_attributes');
    }
};
