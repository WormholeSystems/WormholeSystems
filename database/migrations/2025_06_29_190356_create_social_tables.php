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
        Schema::create('bloodlines', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name');
            $table->text('description');
            $table->foreignId('race_id')->constrained();
            $table->foreignId('ship_type_id')->nullable()->constrained('types');
            $table->unsignedBigInteger('willpower');
            $table->unsignedBigInteger('perception');
            $table->unsignedBigInteger('charisma');
            $table->unsignedBigInteger('intelligence');
            $table->unsignedBigInteger('memory');
            $table->timestamps();
        });

        Schema::create('corporations', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->nullable()->index();
            $table->string('ticker')->nullable()->index();
            $table->unsignedBigInteger('ceo_id')->nullable();
            $table->unsignedBigInteger('creator_id')->nullable();
            $table->unsignedBigInteger('faction_id')->nullable();
            $table->unsignedBigInteger('alliance_id')->nullable();
            $table->foreignId('home_station_id')->nullable()->constrained('stations');
            $table->unsignedBigInteger('member_count')->nullable();
            $table->unsignedBigInteger('shares')->nullable();
            $table->dateTime('date_founded')->nullable();
            $table->text('description')->nullable();
            $table->text('url')->nullable();
            $table->float('tax_rate')->nullable();
            $table->boolean('war_eligible')->nullable()->index();
            $table->boolean('npc')->default(false)->index();
            $table->dateTime('last_updated')->nullable()->index();
            $table->timestamps();
        });

        Schema::create('factions', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name');
            $table->text('description');
            $table->foreignId('corporation_id')->nullable()->constrained();
            $table->foreignId('militia_corporation_id')->nullable()->constrained('corporations');
            $table->foreignId('solarsystem_id')->nullable()->constrained();
            $table->double('size_factor');
            $table->unsignedBigInteger('station_count');
            $table->unsignedBigInteger('station_system_count');
            $table->boolean('is_unique');
            $table->timestamps();
        });

        Schema::create('characters', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->foreignId('user_id')->nullable()->constrained()->nullOnDelete();
            $table->text('character_owner_hash')->nullable();
            $table->string('name')->nullable();
            $table->text('description')->nullable();
            $table->foreignId('race_id')->nullable();
            $table->foreignId('bloodline_id')->nullable()->constrained();
            $table->foreignId('corporation_id')->nullable()->constrained();
            $table->foreignId('faction_id')->nullable()->constrained();
            $table->unsignedBigInteger('alliance_id')->nullable();
            $table->double('security_status')->nullable();
            $table->string('gender')->nullable();
            $table->dateTime('birthday')->nullable();
            $table->string('title')->nullable();
            $table->timestamps();
        });

        Schema::create('alliances', function (Blueprint $table): void {
            $table->unsignedBigInteger('id')->primary();
            $table->string('name')->nullable();
            $table->string('ticker')->nullable();
            $table->foreignId('creator_id')->nullable()->constrained('characters');
            $table->foreignId('creator_corporation_id')->nullable()->constrained('corporations');
            $table->foreignId('faction_id')->nullable()->constrained();
            $table->dateTime('date_founded')->nullable();
            $table->dateTime('last_updated')->nullable()->index();
            $table->timestamps();
        });

        Schema::table('corporations', function (Blueprint $table): void {
            $table->foreign('ceo_id')->references('id')->on('characters');
            $table->foreign('creator_id')->references('id')->on('characters');
            $table->foreign('faction_id')->references('id')->on('factions');
            $table->foreign('alliance_id')->references('id')->on('alliances');
        });

        Schema::table('characters', function (Blueprint $table): void {
            $table->foreign('alliance_id')->references('id')->on('alliances');
        });

        Schema::table('bloodlines', function (Blueprint $table): void {
            $table->foreignId('corporation_id')->nullable()->constrained();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('stations');
    }
};
