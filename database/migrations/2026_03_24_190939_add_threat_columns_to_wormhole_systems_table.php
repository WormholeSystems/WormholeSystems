<?php

declare(strict_types=1);

use App\Enums\ThreatLevel;
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
        Schema::table('wormhole_systems', function (Blueprint $table): void {
            $table->string('threat_level')->default(ThreatLevel::Unknown->value)->after('class');
            $table->json('threat_data')->nullable()->after('threat_level');
            $table->timestamp('threat_analyzed_at')->nullable()->after('threat_data');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('wormhole_systems', function (Blueprint $table): void {
            $table->dropColumn(['threat_level', 'threat_data', 'threat_analyzed_at']);
        });
    }
};
