<?php

declare(strict_types=1);

use App\Enums\MapSolarsystemStatus;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

/**
 * Splits MapSolarsystem into a placement table (map_solarsystems: position/alias/pinned)
 * and a persistent details table (map_solarsystem_details: occupier/status/notes/audits).
 *
 * Placement rows can now be hard-deleted on removal so connections and signatures cascade,
 * while per-map intel survives. Today's "removed" rows (null position) become details rows
 * with no placement.
 *
 * Targets MySQL (test + prod). Destructive: drops columns and deletes null-position rows.
 */
return new class extends Migration
{
    public function up(): void
    {
        $this->deduplicatePlacements();

        Schema::create('map_solarsystem_details', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('solarsystem_id')->constrained('solarsystems')->cascadeOnDelete()->cascadeOnUpdate();
            $table->string('occupier_alias')->nullable()->index();
            $table->string('status')->default(MapSolarsystemStatus::Unknown->value)->index();
            $table->text('notes')->nullable();
            $table->unique(['map_id', 'solarsystem_id']);
            $table->timestamps();
        });

        // One details row per existing placement (map_solarsystems is already unique on
        // (map_id, solarsystem_id), so this is a clean 1:1 copy).
        DB::statement('
            INSERT INTO map_solarsystem_details
                (map_id, solarsystem_id, occupier_alias, status, notes, created_at, updated_at)
            SELECT map_id, solarsystem_id, occupier_alias, status, notes, created_at, updated_at
            FROM map_solarsystems
        ');

        Schema::table('map_solarsystems', function (Blueprint $table): void {
            $table->foreignId('map_solarsystem_details_id')
                ->nullable()
                ->after('solarsystem_id')
                ->constrained('map_solarsystem_details')
                ->cascadeOnDelete();
        });

        DB::statement('
            UPDATE map_solarsystems ms
            JOIN map_solarsystem_details d
              ON d.map_id = ms.map_id AND d.solarsystem_id = ms.solarsystem_id
            SET ms.map_solarsystem_details_id = d.id
        ');

        // Repoint audit history from the placement model to the details model before any
        // placement rows are deleted.
        DB::statement('
            UPDATE audits a
            JOIN map_solarsystems ms ON a.auditable_id = ms.id
            SET a.auditable_id = ms.map_solarsystem_details_id,
                a.auditable_type = ?
            WHERE a.auditable_type = ?
        ', [App\Models\MapSolarsystemDetails::class, App\Models\MapSolarsystem::class]);

        $this->repointHomeToSolarsystem();

        Schema::table('map_solarsystems', function (Blueprint $table): void {
            $table->dropColumn(['occupier_alias', 'status', 'notes']);
        });

        // Remove "off-map" placements; their details survive and their connections /
        // signatures cascade away (this also cleans up any pre-existing orphans).
        DB::table('map_solarsystems')->whereNull('position_x')->delete();

        // A placement now always represents a system on the canvas, so it always has a
        // position and a details link.
        Schema::table('map_solarsystems', function (Blueprint $table): void {
            $table->foreignId('map_solarsystem_details_id')->nullable(false)->change();
            $table->bigInteger('position_x')->nullable(false)->change();
            $table->bigInteger('position_y')->nullable(false)->change();
        });
    }

    public function down(): void
    {
        Schema::table('map_solarsystems', function (Blueprint $table): void {
            $table->bigInteger('position_x')->nullable()->change();
            $table->bigInteger('position_y')->nullable()->change();
            $table->string('occupier_alias')->nullable()->index();
            $table->string('status')->default(MapSolarsystemStatus::Unknown->value)->index();
            $table->text('notes')->nullable();
        });

        DB::statement('
            UPDATE map_solarsystems ms
            JOIN map_solarsystem_details d ON d.id = ms.map_solarsystem_details_id
            SET ms.occupier_alias = d.occupier_alias,
                ms.status = d.status,
                ms.notes = d.notes
        ');

        DB::statement('
            UPDATE audits a
            JOIN map_solarsystems ms ON a.auditable_id = ms.map_solarsystem_details_id
            SET a.auditable_id = ms.id,
                a.auditable_type = ?
            WHERE a.auditable_type = ?
        ', [App\Models\MapSolarsystem::class, App\Models\MapSolarsystemDetails::class]);

        $this->restoreHomeToPlacement();

        Schema::table('map_solarsystems', function (Blueprint $table): void {
            $table->dropConstrainedForeignId('map_solarsystem_details_id');
        });

        Schema::dropIfExists('map_solarsystem_details');
    }

    /**
     * Collapse duplicate placements that share a (map_id, solarsystem_id) before the split
     * adds the unique details row per pair. Such duplicates only exist from historic
     * double-adds (the old schema never enforced uniqueness); the canonical row is the
     * lowest id, and any connections / signatures / home reference on a duplicate is moved
     * onto it so nothing is lost. Neither map_connections nor signatures has a unique
     * constraint beyond its primary key, so these repoints cannot collide.
     */
    private function deduplicatePlacements(): void
    {
        $keeper = '
            (SELECT map_id, solarsystem_id, MIN(id) AS keeper_id
             FROM map_solarsystems
             GROUP BY map_id, solarsystem_id) k
        ';

        DB::statement("
            UPDATE map_connections mc
            JOIN map_solarsystems ms ON ms.id = mc.from_map_solarsystem_id
            JOIN {$keeper} ON k.map_id = ms.map_id AND k.solarsystem_id = ms.solarsystem_id
            SET mc.from_map_solarsystem_id = k.keeper_id
            WHERE mc.from_map_solarsystem_id <> k.keeper_id
        ");

        DB::statement("
            UPDATE map_connections mc
            JOIN map_solarsystems ms ON ms.id = mc.to_map_solarsystem_id
            JOIN {$keeper} ON k.map_id = ms.map_id AND k.solarsystem_id = ms.solarsystem_id
            SET mc.to_map_solarsystem_id = k.keeper_id
            WHERE mc.to_map_solarsystem_id <> k.keeper_id
        ");

        DB::statement("
            UPDATE signatures s
            JOIN map_solarsystems ms ON ms.id = s.map_solarsystem_id
            JOIN {$keeper} ON k.map_id = ms.map_id AND k.solarsystem_id = ms.solarsystem_id
            SET s.map_solarsystem_id = k.keeper_id
            WHERE s.map_solarsystem_id <> k.keeper_id
        ");

        DB::statement("
            UPDATE maps m
            JOIN map_solarsystems ms ON ms.id = m.home_solarsystem_id
            JOIN {$keeper} ON k.map_id = ms.map_id AND k.solarsystem_id = ms.solarsystem_id
            SET m.home_solarsystem_id = k.keeper_id
            WHERE m.home_solarsystem_id <> k.keeper_id
        ");

        DB::statement("
            DELETE ms FROM map_solarsystems ms
            JOIN {$keeper} ON k.map_id = ms.map_id AND k.solarsystem_id = ms.solarsystem_id
            WHERE ms.id <> k.keeper_id
        ");
    }

    /**
     * Convert maps.home_solarsystem_id from a map_solarsystems FK to a solarsystems FK so the
     * home survives a system being removed and re-added.
     */
    private function repointHomeToSolarsystem(): void
    {
        /** @var array<int, int> $oldHomes map_id => old map_solarsystem id */
        $oldHomes = DB::table('maps')->whereNotNull('home_solarsystem_id')
            ->pluck('home_solarsystem_id', 'id')->all();

        Schema::table('maps', function (Blueprint $table): void {
            $table->dropConstrainedForeignId('home_solarsystem_id');
        });

        Schema::table('maps', function (Blueprint $table): void {
            $table->foreignId('home_solarsystem_id')->nullable()->constrained('solarsystems')->nullOnDelete();
        });

        if ($oldHomes === []) {
            return;
        }

        $placementToSolarsystem = DB::table('map_solarsystems')
            ->whereIn('id', array_values($oldHomes))
            ->pluck('solarsystem_id', 'id');

        foreach ($oldHomes as $mapId => $oldPlacementId) {
            $solarsystemId = $placementToSolarsystem[$oldPlacementId] ?? null;

            if ($solarsystemId !== null) {
                DB::table('maps')->where('id', $mapId)->update(['home_solarsystem_id' => $solarsystemId]);
            }
        }
    }

    private function restoreHomeToPlacement(): void
    {
        /** @var array<int, int> $oldHomes map_id => solarsystem_id */
        $oldHomes = DB::table('maps')->whereNotNull('home_solarsystem_id')
            ->pluck('home_solarsystem_id', 'id')->all();

        Schema::table('maps', function (Blueprint $table): void {
            $table->dropConstrainedForeignId('home_solarsystem_id');
        });

        Schema::table('maps', function (Blueprint $table): void {
            $table->foreignId('home_solarsystem_id')->nullable()->constrained('map_solarsystems')->nullOnDelete();
        });

        foreach ($oldHomes as $mapId => $solarsystemId) {
            $placementId = DB::table('map_solarsystems')
                ->where('map_id', $mapId)
                ->where('solarsystem_id', $solarsystemId)
                ->value('id');

            if ($placementId !== null) {
                DB::table('maps')->where('id', $mapId)->update(['home_solarsystem_id' => $placementId]);
            }
        }
    }
};
