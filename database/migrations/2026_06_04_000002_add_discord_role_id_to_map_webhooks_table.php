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
            $table->string('discord_role_id')->nullable()->after('discord_webhook_url');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_webhooks', function (Blueprint $table): void {
            $table->dropColumn('discord_role_id');
        });
    }
};
