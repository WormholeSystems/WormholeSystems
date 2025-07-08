<?php

use App\Enums\MassStatus;
use App\Enums\ShipSize;
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
        Schema::create('map_connections', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')
                ->constrained('maps')
                ->cascadeOnDelete()
                ->cascadeOnUpdate();
            $table->foreignId('from_map_solarsystem_id')
                ->constrained('map_solarsystems')
                ->cascadeOnDelete()
                ->cascadeOnUpdate();
            $table->foreignId('to_map_solarsystem_id')
                ->constrained('map_solarsystems')
                ->cascadeOnDelete()
                ->cascadeOnUpdate();
            $table->foreignId('wormhole_id')->nullable()->constrained('wormholes')->cascadeOnDelete()->cascadeOnUpdate();
            $table->string('mass_status')->default(MassStatus::Fresh->value)->index();
            $table->string('ship_size')->default(ShipSize::Large->value)->index();
            $table->boolean('is_eol')->default(false)->index();
            $table->dateTime('connected_at')->nullable();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('map_connections');
    }
};
