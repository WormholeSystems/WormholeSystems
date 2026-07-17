<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('wormholes', function (Blueprint $table): void {
            $table->dropColumn('ship_size');
        });
    }

    public function down(): void
    {
        Schema::table('wormholes', function (Blueprint $table): void {
            $table->string('ship_size')->nullable()->index();
        });
    }
};
