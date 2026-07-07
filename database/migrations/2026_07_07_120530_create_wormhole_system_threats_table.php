<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /** Normalizes `wormhole_systems.threat_data` into rows, copying existing entries first. */
    public function up(): void
    {
        Schema::create('wormhole_system_threats', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('wormhole_system_id')->constrained('wormhole_systems')->cascadeOnDelete()->cascadeOnUpdate();
            $table->unsignedBigInteger('entity_id');
            $table->string('entity_type');
            $table->string('name');
            $table->unsignedInteger('kills')->default(0);

            $table->unique(['wormhole_system_id', 'entity_type', 'entity_id'], 'wormhole_system_threats_entity_unique');
        });

        DB::table('wormhole_systems')
            ->whereNotNull('threat_data')
            ->chunkById(500, function ($systems): void {
                $rows = [];

                foreach ($systems as $system) {
                    foreach (json_decode((string) $system->threat_data, true) ?: [] as $entity) {
                        $rows[] = [
                            'wormhole_system_id' => $system->id,
                            'entity_id' => (int) ($entity['id'] ?? 0),
                            'entity_type' => (string) ($entity['type'] ?? 'unknown'),
                            'name' => (string) ($entity['name'] ?? 'Unknown entity'),
                            'kills' => (int) ($entity['kills'] ?? 0),
                        ];
                    }
                }

                if ($rows !== []) {
                    DB::table('wormhole_system_threats')->insert($rows);
                }
            });

        Schema::table('wormhole_systems', function (Blueprint $table): void {
            $table->dropColumn('threat_data');
        });
    }

    /** The daily analysis regenerates threat data, so the rollback does not backfill the JSON. */
    public function down(): void
    {
        Schema::table('wormhole_systems', function (Blueprint $table): void {
            $table->json('threat_data')->nullable()->after('threat_level');
        });

        Schema::dropIfExists('wormhole_system_threats');
    }
};
