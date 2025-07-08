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
        Schema::create('groups', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->index();
            $table->foreignId('category_id')->index()->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('icon_id')->nullable()->index()->constrained()->nullOnDelete()->cascadeOnUpdate();
            $table->boolean('published')->default(true)->index();
            $table->boolean('use_base_price')->default(false)->index();
            $table->boolean('anchored')->default(false)->index();
            $table->boolean('anchorable')->default(false)->index();
            $table->boolean('fittable_non_singleton')->default(false)->index();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('groups');
    }
};
