<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('meta_groups', function (Blueprint $table) {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->index();
            $table->string('description')->nullable();
            $table->foreignId('icon_id')->nullable()->constrained()->nullOnDelete()->cascadeOnUpdate();
            $table->string('icon_suffix')->nullable();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('meta_groups');
    }
};
