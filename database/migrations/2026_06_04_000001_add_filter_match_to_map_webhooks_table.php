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
            $table->string('filter_match')->default('any')->after('filters');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_webhooks', function (Blueprint $table): void {
            $table->dropColumn('filter_match');
        });
    }
};
