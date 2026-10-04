<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Query\Expression;
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
        Schema::table('maps', function (Blueprint $table): void {
            $table->json('rally_solarsystem_ids')->default(new Expression('(JSON_ARRAY())'))->after('rally_solarsystem_id');
        });

        DB::table('maps')
            ->whereNotNull('rally_solarsystem_id')
            ->update(['rally_solarsystem_ids' => DB::raw('JSON_ARRAY(rally_solarsystem_id)')]);

        Schema::table('maps', function (Blueprint $table): void {
            $table->dropConstrainedForeignId('rally_solarsystem_id');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('maps', function (Blueprint $table): void {
            $table->foreignId('rally_solarsystem_id')->nullable()->after('home_solarsystem_id')->constrained('solarsystems')->nullOnDelete();
        });

        DB::table('maps')
            ->whereRaw('JSON_LENGTH(rally_solarsystem_ids) > 0')
            ->update(['rally_solarsystem_id' => DB::raw("JSON_EXTRACT(rally_solarsystem_ids, '$[0]')")]);

        Schema::table('maps', function (Blueprint $table): void {
            $table->dropColumn('rally_solarsystem_ids');
        });
    }
};
