<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     *
     * A null user_id marks a shared row; personal rows belong to their user. The new unique
     * index is added before the old one is dropped: the map_id foreign key needs an index
     * starting with map_id at all times.
     */
    public function up(): void
    {
        Schema::table('map_route_solarsystems', function (Blueprint $table): void {
            $table->foreignId('user_id')->nullable()->after('map_id')->constrained()->cascadeOnDelete();
            $table->unique(['map_id', 'user_id', 'solarsystem_id']);
        });

        Schema::table('map_route_solarsystems', function (Blueprint $table): void {
            $table->dropUnique(['map_id', 'solarsystem_id']);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        DB::table('map_route_solarsystems')->whereNotNull('user_id')->delete();

        Schema::table('map_route_solarsystems', function (Blueprint $table): void {
            $table->unique(['map_id', 'solarsystem_id']);
        });

        Schema::table('map_route_solarsystems', function (Blueprint $table): void {
            $table->dropUnique(['map_id', 'user_id', 'solarsystem_id']);
            $table->dropConstrainedForeignId('user_id');
        });
    }
};
