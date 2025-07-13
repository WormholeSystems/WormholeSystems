<?php

use App\Enums\MapSolarsystemStatus;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('map_solarsystems', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('solarsystem_id')->constrained('solarsystems')->cascadeOnDelete()->cascadeOnUpdate();
            $table->string('alias')->nullable()->index();
            $table->string('occupier_alias')->nullable()->index();
            $table->bigInteger('position_x')->nullable();
            $table->bigInteger('position_y')->nullable();
            $table->string('status')->default(MapSolarsystemStatus::Unknown)->index();
            $table->boolean('pinned')->default(false);
            $table->text('notes')->nullable();
            $table->unique(['map_id', 'solarsystem_id']);
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('map_solarsystems');
    }
};
