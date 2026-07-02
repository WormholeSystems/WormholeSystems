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
        Schema::table('map_alerts', function (Blueprint $table): void {
            $table->string('ship_type')->nullable()->after('target_solarsystem_id');
            $table->unsignedTinyInteger('jdc_level')->nullable()->after('ship_type');
            $table->boolean('include_highsec')->default(false)->after('jdc_level');
            $table->unsignedTinyInteger('max_jumps')->nullable()->change();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_alerts', function (Blueprint $table): void {
            $table->dropColumn(['ship_type', 'jdc_level', 'include_highsec']);
            $table->unsignedTinyInteger('max_jumps')->default(5)->change();
        });
    }
};
