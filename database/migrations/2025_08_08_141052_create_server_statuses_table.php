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
        Schema::create('server_statuses', function (Blueprint $table): void {
            $table->id();
            $table->unsignedBigInteger('players');
            $table->string('server_version');
            $table->timestamp('start_time')->nullable();
            $table->boolean('vip')->default(false);
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('server_statuses');
    }
};
