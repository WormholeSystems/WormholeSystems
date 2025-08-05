<?php

use App\Enums\KillmailFilter;
use App\Enums\MassStatus;
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
        Schema::table('map_user_settings', function (Blueprint $table): void {
            $table->boolean('is_tracking')->default(false);
            $table->boolean('route_allow_eol')->default(false);
            $table->string('route_allow_mass_status')->default(MassStatus::Reduced);
            $table->boolean('route_use_evescout')->default(false);
            $table->string('killmail_filter')->default(KillmailFilter::All);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_user_settings', function (Blueprint $table): void {
            $table->dropColumn([
                'is_tracking',
                'route_allow_eol',
                'route_allow_mass_status',
                'route_use_evescout',
                'killmail_filter',
            ]);
        });
    }
};
