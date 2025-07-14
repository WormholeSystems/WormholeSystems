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
        Schema::create('effects', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->text('description')->nullable();
            $table->boolean('disallow_auto_repeat')->default(false);
            $table->bigInteger('discharge_attribute_id')->nullable();
            $table->string('display_name')->nullable();
            $table->bigInteger('duration_attribute_id')->nullable();
            $table->bigInteger('effect_category')->nullable();
            $table->boolean('electronic_chance')->default(false);
            $table->bigInteger('falloff_attribute_id')->nullable();
            $table->bigInteger('icon_id')->nullable();
            $table->boolean('is_assistance')->default(false);
            $table->boolean('is_offensive')->default(false);
            $table->boolean('is_warp_safe')->default(false);
            $table->string('name')->nullable();
            $table->bigInteger('post_expression')->nullable();
            $table->bigInteger('pre_expression')->nullable();
            $table->boolean('published')->default(false);
            $table->bigInteger('range_attribute_id')->nullable();
            $table->boolean('range_chance')->default(false);
            $table->bigInteger('tracking_speed_attribute_id')->nullable();
            $table->boolean('propulsion_chance')->default(false);
            $table->bigInteger('resistance_attribute_id')->nullable();
            $table->bigInteger('fitting_usage_chance_attribute_id')->nullable();
            $table->string('sfx_name')->nullable();
            $table->string('distribution')->nullable();
            $table->timestamps();
        });

        Schema::create('effect_modifiers', function (Blueprint $table): void {
            $table->id();
            $table->unsignedBigInteger('effect_id');
            $table->string('domain');
            $table->string('func');
            $table->unsignedBigInteger('modified_attribute_id')->nullable();
            $table->unsignedBigInteger('modifying_attribute_id')->nullable();
            $table->bigInteger('operator')->nullable();
            $table->foreign('effect_id')->references('id')->on('effects')->cascadeOnDelete()->cascadeOnUpdate();
            $table->foreignId('group_id')->nullable()->constrained()->cascadeOnDelete()->cascadeOnUpdate();
            $table->unsignedBigInteger('skill_type_id')->nullable();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('effect_modifiers');
        Schema::dropIfExists('effects');
    }
};
