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
        Schema::table('map_webhooks', function (Blueprint $table): void {
            $table->json('filters')->nullable()->after('max_jumps');
        });

        Schema::table('map_webhooks', function (Blueprint $table): void {
            $table->dropForeign(['target_solarsystem_id']);
            $table->unsignedBigInteger('target_solarsystem_id')->nullable()->change();
            $table->foreign('target_solarsystem_id')->references('id')->on('solarsystems')->onDelete('cascade');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_webhooks', function (Blueprint $table): void {
            $table->dropForeign(['target_solarsystem_id']);
            $table->unsignedBigInteger('target_solarsystem_id')->nullable(false)->change();
            $table->foreign('target_solarsystem_id')->references('id')->on('solarsystems')->onDelete('cascade');
        });

        Schema::table('map_webhooks', function (Blueprint $table): void {
            $table->dropColumn('filters');
        });
    }
};
