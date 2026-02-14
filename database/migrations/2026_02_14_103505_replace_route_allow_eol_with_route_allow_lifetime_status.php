<?php

declare(strict_types=1);

use App\Enums\LifetimeStatus;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::table('map_user_settings', function (Blueprint $table): void {
            $table->string('route_allow_lifetime_status')
                ->default(LifetimeStatus::Critical->value)
                ->after('route_allow_eol');
        });

        // Migrate existing data: true (allow EOL) → critical, false (disallow EOL) → healthy
        DB::table('map_user_settings')
            ->where('route_allow_eol', true)
            ->update(['route_allow_lifetime_status' => LifetimeStatus::Critical->value]);

        DB::table('map_user_settings')
            ->where('route_allow_eol', false)
            ->update(['route_allow_lifetime_status' => LifetimeStatus::Healthy->value]);

        Schema::table('map_user_settings', function (Blueprint $table): void {
            $table->dropColumn('route_allow_eol');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_user_settings', function (Blueprint $table): void {
            $table->boolean('route_allow_eol')->default(true)->after('is_tracking');
        });

        DB::table('map_user_settings')
            ->where('route_allow_lifetime_status', LifetimeStatus::Healthy->value)
            ->update(['route_allow_eol' => false]);

        DB::table('map_user_settings')
            ->whereIn('route_allow_lifetime_status', [LifetimeStatus::EndOfLife->value, LifetimeStatus::Critical->value])
            ->update(['route_allow_eol' => true]);

        Schema::table('map_user_settings', function (Blueprint $table): void {
            $table->dropColumn('route_allow_lifetime_status');
        });
    }
};
