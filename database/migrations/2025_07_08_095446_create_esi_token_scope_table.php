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
        Schema::create('esi_token_scope', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('esi_token_id')->constrained()->cascadeOnDelete();
            $table->foreignId('esi_scope_id')->constrained()->cascadeOnDelete();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('esi_token_scope');
    }
};
