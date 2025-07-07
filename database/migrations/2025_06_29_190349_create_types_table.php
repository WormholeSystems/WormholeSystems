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
        Schema::create('types', function (Blueprint $table) {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->index();
            $table->text('description')->nullable();
            $table->foreignId('graphic_id')->nullable()->constrained()->nullOnDelete()->cascadeOnUpdate();
            $table->foreignId('group_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('icon_id')->nullable()->constrained()->nullOnDelete()->cascadeOnUpdate();
            $table->foreignId('market_group_id')->nullable()->constrained()->nullOnDelete()->cascadeOnUpdate();
            $table->foreignId('meta_group_id')->nullable()->constrained()->nullOnDelete()->cascadeOnUpdate();
            $table->foreignId('race_id')->nullable()->constrained()->nullOnDelete()->cascadeOnUpdate();
            $table->boolean('published')->default(true)->index();
            $table->double('capacity')->nullable();
            $table->double('mass')->nullable();
            $table->double('base_price')->nullable();
            $table->double('volume')->nullable();
            $table->double('packaged_volume')->nullable();
            $table->double('radius')->nullable();
            $table->double('portion_size')->nullable();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('types');
    }
};
