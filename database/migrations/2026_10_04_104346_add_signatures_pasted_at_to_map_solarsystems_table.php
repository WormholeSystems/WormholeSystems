<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('map_solarsystems', function (Blueprint $table): void {
            $table->timestamp('signatures_pasted_at')->nullable()->after('pinned');
        });
    }

    public function down(): void
    {
        Schema::table('map_solarsystems', function (Blueprint $table): void {
            $table->dropColumn('signatures_pasted_at');
        });
    }
};
