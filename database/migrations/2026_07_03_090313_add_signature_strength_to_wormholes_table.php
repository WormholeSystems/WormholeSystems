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
        Schema::table('wormholes', function (Blueprint $table): void {
            $table->decimal('signature_strength', 5, 2)->nullable()->after('leads_to');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('wormholes', function (Blueprint $table): void {
            $table->dropColumn('signature_strength');
        });
    }
};
