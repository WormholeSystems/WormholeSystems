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
        Schema::table('maps', function (Blueprint $table): void {
            $table->string('bookmark_format_wormhole')->default('{alias} {sig} {class}');
            $table->string('bookmark_format_kspace')->default('{alias} {class} {sig} {name} {region}');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('maps', function (Blueprint $table): void {
            $table->dropColumn(['bookmark_format_wormhole', 'bookmark_format_kspace']);
        });
    }
};
