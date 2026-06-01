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
        Schema::create('map_webhooks', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')->constrained()->onDelete('cascade');
            $table->string('name');
            $table->text('discord_webhook_url');
            $table->string('type')->default('proximity')->index();
            $table->foreignId('target_solarsystem_id')->constrained('solarsystems')->onDelete('cascade');
            $table->unsignedTinyInteger('max_jumps');
            $table->boolean('is_active')->default(true)->index();
            $table->timestamp('last_fired_at')->nullable();

            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('map_webhooks');
    }
};
