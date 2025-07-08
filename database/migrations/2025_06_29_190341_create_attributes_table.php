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
        Schema::create('attributes', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->index();
            $table->string('display_name');
            $table->text('description');
            $table->decimal('default_value', 50, 2)->default(0);
            $table->foreignId('unit_id')->nullable()->constrained()->nullOnDelete()->cascadeOnUpdate();
            $table->foreignId('icon_id')->nullable()->constrained()->nullOnDelete()->cascadeOnUpdate();
            $table->boolean('high_is_good')->default(true)->index();
            $table->boolean('published')->default(true)->index();
            $table->boolean('stackable')->default(false)->index();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('attributes');
    }
};
