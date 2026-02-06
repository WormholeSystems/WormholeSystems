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
        Schema::table('stations', function (Blueprint $table): void {
            $table->foreignId('operation_id')->nullable()->after('type_id')->constrained('station_operations')->nullOnDelete();
            $table->foreignId('owner_id')->nullable()->after('operation_id');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('stations', function (Blueprint $table): void {
            $table->dropForeign(['operation_id']);
            $table->dropColumn(['operation_id', 'owner_id']);
        });
    }
};
